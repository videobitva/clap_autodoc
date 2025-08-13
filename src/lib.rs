use darling::{ast::NestedMeta, FromMeta};
use heck::{
    ToKebabCase, ToLowerCamelCase, ToPascalCase, ToShoutyKebabCase, ToShoutySnakeCase, ToSnakeCase,
};
use indexmap::IndexMap;
use lazy_static::lazy_static;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use std::collections::HashMap;
use std::fs;
use std::path::Path as StdPath;
use std::sync::RwLock;
use syn::{
    parse::Parse, parse::ParseStream, parse_macro_input,
    Attribute, Data, DataStruct, DeriveInput, Expr, ExprLit, Field, Fields, Lit, Meta,
    MetaList, MetaNameValue, Path, Type, TypePath,
};
use tabled::{Table, Tabled};

// Global registry for struct definitions and file-specific pending generations
lazy_static! {
    static ref STRUCT_REGISTRY: RwLock<HashMap<String, StructInfo>> = RwLock::new(HashMap::new());
    static ref FILE_PENDING_GENERATIONS: RwLock<HashMap<String, Vec<PendingGeneration>>> = RwLock::new(HashMap::new());
}

/// Information about a pending documentation generation
#[derive(Debug, Clone)]
struct PendingGeneration {
    struct_info: StructInfo,
    args: ConfigDocsArgs,
}

/// Configuration documentation generator attribute macro
///
/// Usage:
/// ```rust
/// #[generate(target = "README.md")]
/// #[generate(target = "README.md", format = "flat")]
/// #[generate(target = "README.md", format = "grouped")]
/// ```
#[proc_macro_attribute]
pub fn generate(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let attr_args = parse_macro_input!(args as AttributeArgs);

    let args = match ConfigDocsArgs::from_list(&attr_args) {
        Ok(args) => args,
        Err(err) => return err.write_errors().into(),
    };

    match generate_config_docs(&input, &args) {
        Ok(result) => result,
        Err(err) => err.to_compile_error().into(),
    }
}

/// Custom parsing for attribute arguments
struct AttributeArgs(Vec<NestedMeta>);

impl Parse for AttributeArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Vec::new();

        while !input.is_empty() {
            let meta = input.parse::<Meta>()?;
            args.push(NestedMeta::Meta(meta));

            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(AttributeArgs(args))
    }
}

impl std::ops::Deref for AttributeArgs {
    type Target = Vec<NestedMeta>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Registration macro for nested structs
///
/// Usage:
/// ```rust
/// use clap_autodoc::register;
///
/// #[register]
/// pub struct DatabaseConfig {
///     // fields...
/// }
/// ```
#[proc_macro_attribute]
pub fn register(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match register_struct_definition(&input) {
        Ok(result) => result,
        Err(err) => err.to_compile_error().into(),
    }
}

/// Arguments for the generate attribute
#[derive(Debug, Clone, FromMeta)]
struct ConfigDocsArgs {
    target: String,
    #[darling(default = "OutputFormat::default")]
    format: OutputFormat,
}

/// Main function to generate configuration documentation with smart dependency resolution
fn generate_config_docs(input: &DeriveInput, args: &ConfigDocsArgs) -> syn::Result<TokenStream> {
    let struct_info = parse_struct_info(input)?;

    if can_generate_immediately(&struct_info)? {
        let expanded_struct_info = expand_nested_structs(struct_info)?;

        let markdown_table = generate_markdown_table(&expanded_struct_info, args)?;

        update_target_file(&args.target, &markdown_table)?;
    } else {
        let mut file_pending = FILE_PENDING_GENERATIONS.write().unwrap();
        file_pending
            .entry(args.target.clone())
            .or_insert_with(Vec::new)
            .push(PendingGeneration {
                struct_info,
                args: args.clone(),
            });
    }

    Ok(quote! { #input }.into())
}

/// Register a struct definition in the global registry
fn register_struct_definition(input: &DeriveInput) -> syn::Result<TokenStream> {
    let struct_info = parse_struct_info(input)?;

    {
        let mut registry = STRUCT_REGISTRY.write().unwrap();
        let struct_name = struct_info.name.clone();
        registry.insert(struct_name, struct_info);
    }

    try_process_pending_generations()?;

    Ok(quote! { #input }.into())
}

/// Check if a struct can be generated immediately (all dependencies are available)
fn can_generate_immediately(struct_info: &StructInfo) -> syn::Result<bool> {
    let registry = STRUCT_REGISTRY.read().unwrap();

    for field in &struct_info.fields {
        if field.clap_attrs.flatten && !registry.contains_key(&field.field_type) {
            return Ok(false); // Missing dependency
        }
    }

    Ok(true)
}

/// Try to process any pending generations that now have all dependencies available
fn try_process_pending_generations() -> syn::Result<()> {
    let mut file_pending = FILE_PENDING_GENERATIONS.write().unwrap();
    
    // Process each target file independently
    for (_target_file, pending_list) in file_pending.iter_mut() {
        let mut remaining_pending = Vec::new();
        
        for pending_gen in pending_list.drain(..) {
            if can_generate_immediately(&pending_gen.struct_info)? {
                let expanded_struct_info = expand_nested_structs(pending_gen.struct_info)?;
                let markdown_table =
                    generate_markdown_table(&expanded_struct_info, &pending_gen.args)?;
                update_target_file(&pending_gen.args.target, &markdown_table)?;
            } else {
                remaining_pending.push(pending_gen);
            }
        }
        
        *pending_list = remaining_pending;
    }
    
    // Remove empty entries to keep the map clean
    file_pending.retain(|_, pending_list| !pending_list.is_empty());

    Ok(())
}

/// Output format for the markdown table
#[derive(Debug, Clone, FromMeta, Default)]
enum OutputFormat {
    #[darling(rename = "flat")]
    #[default]
    Flat,
    #[darling(rename = "grouped")]
    Grouped,
}

/// Information about a struct field
#[derive(Debug, Clone)]
struct FieldInfo {
    name: String,
    field_type: String,
    doc_comment: Option<String>,
    clap_attrs: ClapAttrs,
    group: String,
}

/// Clap attributes for a field 
#[derive(Debug, Clone, Default)]
struct ClapAttrs {
    // Value attributes
    default_value: Option<String>,
    default_value_t: Option<String>,
    
    // Naming attributes
    rename: Option<String>,
    long: Option<String>,
    short: Option<char>,
    
    // Behavioral flags
    flatten: bool,
    required: bool,
    skip: bool,
    
    // Documentation attributes
    help: Option<String>,
    about: Option<String>,
    
    // Environment binding
    env: Option<String>,
}

/// Information about the entire struct
#[derive(Debug, Clone)]
struct StructInfo {
    name: String,
    fields: Vec<FieldInfo>,
    clap_rename_all: Option<CaseStyle>,
}

#[derive(Debug, Clone, Copy)]
enum CaseStyle {
    Snake,
    Camel,
    Pascal,
    Kebab,
    ScreamingSnake,
    ScreamingKebab,
}

impl CaseStyle {
    fn parse(s: &str) -> Option<Self> {
        match s {
            "snake_case" => Some(CaseStyle::Snake),
            "camelCase" => Some(CaseStyle::Camel),
            "PascalCase" => Some(CaseStyle::Pascal),
            "kebab-case" => Some(CaseStyle::Kebab),
            "SCREAMING_SNAKE_CASE" => Some(CaseStyle::ScreamingSnake),
            "SCREAMING-KEBAB-CASE" => Some(CaseStyle::ScreamingKebab),
            _ => None,
        }
    }
}

/// Parse struct information including fields and clap attributes
fn parse_struct_info(input: &DeriveInput) -> syn::Result<StructInfo> {
    let struct_name = input.ident.to_string();

    let clap_rename_all = parse_struct_clap_attrs(&input.attrs)?;

    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => {
            let mut field_infos = Vec::new();
            for field in &fields.named {
                let field_info = parse_field_info(field, &struct_name)?;
                field_infos.push(field_info);
            }
            field_infos
        }
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "only named struct fields are supported",
            ))
        }
    };

    Ok(StructInfo {
        name: struct_name,
        fields,
        clap_rename_all,
    })
}

/// Parse struct-level rename_all clap attribute
fn parse_struct_clap_attrs(attrs: &[Attribute]) -> syn::Result<Option<CaseStyle>> {
    let mut rename_all = None;

    for attr in attrs {
        if attr.path().is_ident("clap") {
            if let Meta::List(list) = &attr.meta {
                let tokens = &list.tokens;
                let tokens_str = tokens.to_string();
                if tokens_str.contains("rename_all") {
                    if let Some(start) = tokens_str.find("rename_all = \"") {
                        let start = start + "rename_all = \"".len();
                        if let Some(end) = tokens_str[start..].find('"') {
                            rename_all = CaseStyle::parse(&tokens_str[start..start + end]);
                        }
                    }
                }
            }
        }
    }

    Ok(rename_all)
}

/// Parse individual field information
fn parse_field_info(field: &Field, parent_struct: &str) -> syn::Result<FieldInfo> {
    let field_name = field.ident.as_ref().unwrap().to_string();
    let field_type = type_to_string(&field.ty);
    let doc_comment = extract_doc_comment(&field.attrs);
    let clap_attrs = parse_field_clap_attrs(&field.attrs)?;

    let group = if clap_attrs.flatten {
        extract_type_name(&field.ty).unwrap_or_else(|| "Unknown".to_string())
    } else {
        parent_struct.to_string()
    };

    Ok(FieldInfo {
        name: field_name,
        field_type,
        doc_comment,
        clap_attrs,
        group,
    })
}

/// Parse clap attributes for a field
fn parse_field_clap_attrs(attrs: &[Attribute]) -> syn::Result<ClapAttrs> {
    let mut clap_attrs = ClapAttrs::default();
    
    for attr in attrs {
        if attr.path().is_ident("clap") {
            match &attr.meta {
                Meta::List(list) => {
                    parse_clap_meta_list(&mut clap_attrs, list)?;
                }
                Meta::Path(_) => {
                    // Handle #[clap] without arguments - currently no-op
                }
                Meta::NameValue(nv) => {
                    parse_clap_name_value(&mut clap_attrs, nv)?;
                }
            }
        }
    }
    
    Ok(clap_attrs)
}

/// Parse a clap meta list like #[clap(flatten, default_value = "test")]
fn parse_clap_meta_list(attrs: &mut ClapAttrs, list: &MetaList) -> syn::Result<()> {
    let nested_metas = darling::ast::NestedMeta::parse_meta_list(list.tokens.clone())?;
    
    for nested_meta in nested_metas {
        match nested_meta {
            darling::ast::NestedMeta::Meta(meta) => parse_clap_meta(attrs, &meta)?,
            darling::ast::NestedMeta::Lit(lit) => {
                return Err(syn::Error::new_spanned(
                    lit,
                    "unexpected literal in clap attribute"
                ));
            }
        }
    }
    
    Ok(())
}

/// Parse individual clap meta items
fn parse_clap_meta(attrs: &mut ClapAttrs, meta: &Meta) -> syn::Result<()> {
    match meta {
        Meta::Path(path) => parse_clap_flag(attrs, path),
        Meta::NameValue(nv) => parse_clap_name_value_meta(attrs, nv),
        Meta::List(list) => parse_clap_nested_list(attrs, list),
    }
}

/// Parse clap flag attributes like `flatten`, `required`, `skip`
fn parse_clap_flag(attrs: &mut ClapAttrs, path: &Path) -> syn::Result<()> {
    let ident = path.get_ident().ok_or_else(|| {
        syn::Error::new_spanned(path, "expected simple identifier")
    })?;
    
    match ident.to_string().as_str() {
        "flatten" => attrs.flatten = true,
        "required" => attrs.required = true,
        "skip" => attrs.skip = true,
        _ => {}
    }
    
    Ok(())
}

/// Parse clap name-value attributes like `env = "VAR"`, `default_value = "test"`
fn parse_clap_name_value_meta(attrs: &mut ClapAttrs, nv: &MetaNameValue) -> syn::Result<()> {
    let name = nv.path.get_ident().ok_or_else(|| {
        syn::Error::new_spanned(&nv.path, "expected simple identifier")
    })?;
    
    match name.to_string().as_str() {
        "long" => attrs.long = Some(parse_string_value(&nv.value)?),
        "short" => attrs.short = Some(parse_char_value(&nv.value)?),
        "env" => attrs.env = Some(parse_string_value(&nv.value)?),
        "default_value" => attrs.default_value = Some(parse_string_value(&nv.value)?),
        "default_value_t" => attrs.default_value_t = Some(parse_expr_value(&nv.value)?),
        "help" => attrs.help = Some(parse_string_value(&nv.value)?),
        "about" => attrs.about = Some(parse_string_value(&nv.value)?),
        "rename" => attrs.rename = Some(parse_string_value(&nv.value)?),
        _ => {}
    }
    
    Ok(())
}

/// Parse clap name-value from top-level attribute
fn parse_clap_name_value(_attrs: &mut ClapAttrs, _nv: &MetaNameValue) -> syn::Result<()> {
    // Handle cases like #[clap = "some_value"] if needed
    // Currently not used by clap
    Ok(())
}

/// Parse nested clap lists
fn parse_clap_nested_list(_attrs: &mut ClapAttrs, list: &MetaList) -> syn::Result<()> {
    Err(syn::Error::new_spanned(
        list,
        "nested lists not supported in clap attributes"
    ))
}

/// Parse string literal value
fn parse_string_value(expr: &Expr) -> syn::Result<String> {
    match expr {
        Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) => {
            Ok(lit_str.value())
        }
        _ => Err(syn::Error::new_spanned(expr, "expected string literal"))
    }
}

/// Parse character literal value
fn parse_char_value(expr: &Expr) -> syn::Result<char> {
    let s = parse_string_value(expr)?;
    let mut chars = s.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => Ok(c),
        _ => Err(syn::Error::new_spanned(expr, "expected single character"))
    }
}

/// Parse expression value (for default_value_t)
fn parse_expr_value(expr: &Expr) -> syn::Result<String> {
    Ok(quote!(#expr).to_string())
}


/// Extract documentation comment from attributes
fn extract_doc_comment(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Meta::NameValue(MetaNameValue {
                value: Expr::Lit(expr_lit),
                ..
            }) = &attr.meta
            {
                if let Lit::Str(lit_str) = &expr_lit.lit {
                    let comment = lit_str.value().trim().to_string();
                    if !comment.is_empty() {
                        return Some(comment);
                    }
                }
            }
        }
    }
    None
}

/// Convert a Type to a string representation
fn type_to_string(ty: &Type) -> String {
    match ty {
        Type::Path(TypePath { path, .. }) => path
            .segments
            .iter()
            .map(|seg| seg.ident.to_string())
            .collect::<Vec<_>>()
            .join("::"),
        _ => ty.to_token_stream().to_string(),
    }
}

/// Extract the type name from a Type (for group naming)
fn extract_type_name(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(TypePath { path, .. }) => path.segments.last().map(|seg| seg.ident.to_string()),
        _ => None,
    }
}

/// Generate markdown table based on struct information and format
fn generate_markdown_table(
    struct_info: &StructInfo,
    config: &ConfigDocsArgs,
) -> syn::Result<String> {
    match config.format {
        OutputFormat::Flat => generate_flat_table(struct_info),
        OutputFormat::Grouped => generate_grouped_table(struct_info),
    }
}

/// Row for flat table format
#[derive(Tabled)]
struct FlatTableRow {
    #[tabled(rename = "Field Name")]
    field_name: String,
    #[tabled(rename = "Type")]
    field_type: String,
    #[tabled(rename = "Required")]
    required: String,
    #[tabled(rename = "Default")]
    default: String,
    #[tabled(rename = "Details")]
    details: String,
    #[tabled(rename = "Group")]
    group: String,
}

/// Generate flat markdown table with Group column
fn generate_flat_table(struct_info: &StructInfo) -> syn::Result<String> {
    let mut rows = Vec::new();

    for field in &struct_info.fields {
        let field_name = apply_field_name_transformation(&field.name, &struct_info.clap_rename_all);
        let required = if field.clap_attrs.default_value.is_some()
            || field.clap_attrs.default_value_t.is_some()
        {
            "No".to_string()
        } else {
            "Yes".to_string()
        };
        let default = field
            .clap_attrs
            .default_value
            .as_ref()
            .or(field.clap_attrs.default_value_t.as_ref())
            .cloned()
            .unwrap_or_else(|| "-".to_string());
        let details = field
            .doc_comment
            .as_ref()
            .unwrap_or(&"".to_string())
            .clone();

        rows.push(FlatTableRow {
            field_name,
            field_type: field.field_type.clone(),
            required,
            default,
            details,
            group: field.group.clone(),
        });
    }

    let table = Table::new(rows)
        .with(tabled::settings::Style::markdown())
        .to_string();

    Ok(table)
}

/// Row for grouped table format
#[derive(Tabled)]
struct GroupedTableRow {
    #[tabled(rename = "Field Name")]
    field_name: String,
    #[tabled(rename = "Type")]
    field_type: String,
    #[tabled(rename = "Required")]
    required: String,
    #[tabled(rename = "Default")]
    default: String,
    #[tabled(rename = "Details")]
    details: String,
}

/// Generate grouped markdown table with separate sections
fn generate_grouped_table(struct_info: &StructInfo) -> syn::Result<String> {
    let mut groups: IndexMap<String, Vec<&FieldInfo>> = IndexMap::new();

    // Group fields by their group name
    for field in &struct_info.fields {
        groups.entry(field.group.clone()).or_default().push(field);
    }

    let mut result = String::new();

    for (group_name, fields) in groups {
        result.push_str(&format!("## {group_name} Configuration\n\n"));

        let mut rows = Vec::new();

        for field in fields {
            let field_name =
                apply_field_name_transformation(&field.name, &struct_info.clap_rename_all);
            let required = if field.clap_attrs.default_value.is_some()
                || field.clap_attrs.default_value_t.is_some()
            {
                "No".to_string()
            } else {
                "Yes".to_string()
            };
            let default = field
                .clap_attrs
                .default_value
                .as_ref()
                .or(field.clap_attrs.default_value_t.as_ref())
                .cloned()
                .unwrap_or_else(|| "-".to_string());
            let details = field
                .doc_comment
                .as_ref()
                .unwrap_or(&"".to_string())
                .clone();

            rows.push(GroupedTableRow {
                field_name,
                field_type: field.field_type.clone(),
                required,
                default,
                details,
            });
        }

        let table = Table::new(rows)
            .with(tabled::settings::Style::markdown())
            .to_string();

        result.push_str(&table);
        result.push_str("\n\n");
    }

    Ok(result)
}

/// Apply field name transformation based on clap rename_all setting
fn apply_field_name_transformation(field_name: &str, rename_all: &Option<CaseStyle>) -> String {
    match rename_all {
        Some(CaseStyle::Snake) => field_name.to_snake_case(),
        Some(CaseStyle::Camel) => field_name.to_lower_camel_case(),
        Some(CaseStyle::Pascal) => field_name.to_pascal_case(),
        Some(CaseStyle::Kebab) => field_name.to_kebab_case(),
        Some(CaseStyle::ScreamingKebab) => field_name.to_shouty_kebab_case(),
        Some(CaseStyle::ScreamingSnake) => field_name.to_shouty_snake_case(),
        None => field_name.to_owned(),
    }
}

/// Expand nested structs for flattened fields
fn expand_nested_structs(struct_info: StructInfo) -> syn::Result<StructInfo> {
    let mut expanded_fields = Vec::new();

    for field in struct_info.fields {
        if field.clap_attrs.flatten {
            if let Some(nested_struct) = get_registered_struct(&field.field_type) {
                for nested_field in nested_struct.fields {
                    let mut expanded_field = nested_field.clone();
                    expanded_field.group = field.field_type.clone();
                    expanded_field.name = apply_field_name_transformation(
                        &expanded_field.name,
                        &struct_info.clap_rename_all,
                    );

                    expanded_fields.push(expanded_field);
                }
            } else {
                let note = format!(
                    "Note: This field is flattened from {} (not registered)",
                    field.field_type
                );
                let mut expanded_field = field.clone();
                expanded_field.doc_comment = Some(note);
                expanded_fields.push(expanded_field);
            }
        } else {
            expanded_fields.push(field);
        }
    }

    Ok(StructInfo {
        name: struct_info.name,
        fields: expanded_fields,
        clap_rename_all: struct_info.clap_rename_all,
    })
}

/// Get a registered struct from the global registry
fn get_registered_struct(struct_name: &str) -> Option<StructInfo> {
    let registry = STRUCT_REGISTRY.read().unwrap();
    registry.get(struct_name).cloned()
}

/// Update the target file with the generated markdown table
fn update_target_file(target_path: &str, markdown_table: &str) -> syn::Result<()> {
    let start_marker = "[//]: # (CONFIG_DOCS_START)";
    let end_marker = "[//]: # (CONFIG_DOCS_END)";

    let content = if StdPath::new(target_path).exists() {
        fs::read_to_string(target_path).map_err(|e| {
            syn::Error::new(
                Span::call_site(),
                format!("Failed to read file {target_path}: {e}"),
            )
        })?
    } else {
        format!("{start_marker}\n\n{end_marker}")
    };

    // Find the markers and replace content between them
    let updated_content = if let (Some(start_pos), Some(end_pos)) =
        (content.find(start_marker), content.find(end_marker))
    {
        let before = &content[..start_pos + start_marker.len()];
        let after = &content[end_pos..];
        // Ensure there's at least one empty line before and after the table content
        format!("{before}\n\n{markdown_table}\n\n{after}")
    } else {
        // If markers don't exist, append them with the table
        format!("{content}\n{start_marker}\n\n{markdown_table}\n\n{end_marker}",)
    };

    fs::write(target_path, updated_content).map_err(|e| {
        syn::Error::new(
            Span::call_site(),
            format!("Failed to write file {target_path}: {e}"),
        )
    })?;

    Ok(())
}
