/*!
 * # BestBefore
 *
 * A Rust procedural macro for managing code with expiration dates.
 *
 * Technical debt and code deprecation are persistent challenges in software development.
 * As systems evolve, certain implementations become obsolete, APIs change, and better patterns
 * emerge. However, identifying and removing deprecated code is often neglected, leading to
 * maintenance burdens and unexpected bugs.
 *
 * The `bestbefore` macro addresses this challenge by providing a compile-time mechanism to:
 *
 * - Mark code with expiration dates
 * - Enforce deadlines with compiler feedback
 * - Manage technical debt systematically
 *
 * Unlike comments or documentation that can be easily overlooked, this macro integrates with
 * the Rust compiler to actively enforce code maintenance policies.
 *
 * ## Usage
 *
 * ```rust
 * use bestbefore::bestbefore;
 *
 * // Generate a warning if compiled after March 2024
 * #[bestbefore("03.2024")]
 * fn legacy_function() {
 *     // ...
 * }
 *
 * // Generate a warning if compiled after January 2023
 * // Cause a compilation error if compiled after December 2023
 * #[bestbefore("01.2023", expires = "12.2023")]
 * fn very_old_function() {
 *     // ...
 * }
 *
 * // Only specify an expiration date (for code that should just be removed by a deadline)
 * #[bestbefore(expires = "01.2028")]
 * fn expires_only_function() {
 *     // ...
 * }
 *
 * // Add a custom message to the warning
 * #[bestbefore("02.2023", message = "Please use new_api() instead")]
 * fn deprecated_with_message() {
 *     // ...
 * }
 * ```
 *
 * ## Date format
 *
 * The macro uses the "MM.YYYY" format for simplicity, for example:
 * - "03.2024" represents March 2024
 * - "12.2023" represents December 2023
 *
 * The day is assumed to be the first of the month.
 *
 * ## Environment variable
 *
 * You can override the current date by setting the `BESTBEFORE_DATE` environment variable,
 * which is useful for testing. The value should be in the same "MM.YYYY" format.
 */

use chrono::{Datelike, Local, NaiveDate};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use std::env;
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, LitStr, Token};

/// A procedural macro that generates warnings or errors at compile time
/// when the compile date exceeds the specified expiration date.
///
/// # Arguments
///
/// * First positional argument: Optional date string in "MM.YYYY" format for warning threshold
/// * `expires`: Optional date string in "MM.YYYY" format for error threshold
/// * `message`: Optional custom message for warnings/errors
///
/// # Examples
///
/// ```rust
/// use bestbefore::bestbefore;
///
/// // Generate a warning if compiled after March 2024
/// #[bestbefore("03.2024")]
/// fn legacy_function() {
///     // This will generate a warning if compiled after March 2024
/// }
///
/// // Generate a warning if compiled after January 2023
/// // Cause a compilation error if compiled after December 2023
/// #[bestbefore("01.2023", expires="12.2023")]
/// fn very_old_function() {
///     // This will:
///     // - Generate a warning if compiled after March 2023
///     // - Cause a compilation error if compiled after December 2023
/// }
///
/// // Only specify expiration date without a warning date
/// #[bestbefore(expires="01.2028")]
/// fn expires_only_function() {
///     // This will cause a compilation error if compiled after January 2028
///     // without generating warnings before that date
/// }
///
/// // Add a custom message to the warning
/// #[bestbefore("02.2023", message="Please use new_api() instead")]
/// fn deprecated_with_message() {
///     // This will generate a warning with custom message if compiled after March 2024
/// }
/// ```
///
/// # Validation
///
/// The macro validates that if an expiration date is provided, it must be after the warning date.
/// This ensures a logical progression from "should be updated" (warning) to "must be removed" (error).
///
/// # Application Target
///
/// The macro can be applied to any Rust code item, including:
/// - Functions
/// - Modules
/// - Structs
/// - Traits
/// - Implementations
/// - Enums
#[proc_macro_attribute]
pub fn bestbefore(attr: TokenStream, item: TokenStream) -> TokenStream {
    fn compile_error(message: String) -> TokenStream {
        let message = syn::LitStr::new(&message, Span::call_site());
        quote! {
            compile_error!(#message);
            
        }
        .into()
    }

    fn format_date(date: NaiveDate) -> String {
        format!("{:02}.{:02}", date.month(), date.year())
    }

    let attr_args = parse_macro_input!(attr as BestBeforeArgs);
    let input = parse_macro_input!(item as syn::Item);

    let current_date = env::var("BESTBEFORE_DATE")
        .as_deref()
        .map(parse_date)
        .unwrap_or_else(|_| {
            let now = Local::now();
            NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap()
        });

    let item_name = item_name(&input);

    if let Some(expires_date) = attr_args.expires_date {
        if expires_date != attr_args.warning_date && expires_date <= attr_args.warning_date {
            return compile_error(format!(
                "Invalid date: expiration date ({}) must be after warning date ({})",
                format_date(expires_date),
                format_date(attr_args.warning_date)
            ));
        }

        if current_date > expires_date {
            let message = attr_args.message.unwrap_or_else(|| {
                format!(
                    "Code '{}' has expired (after {}): consider removing this code",
                    item_name,
                    format_date(expires_date)
                )
            });
            return compile_error(message);
        }
    }

    let mut result = TokenStream2::new();

    if current_date > attr_args.warning_date {
        let message = attr_args.message.unwrap_or_else(|| {
            format!(
                "Code '{}' past warning date ({}): consider updating or removing this code",
                item_name,
                format_date(attr_args.warning_date)
            )
        });

        let warning = quote! {
            #[warn(deprecated)]
            #[deprecated(note = #message)]
        };

        result.extend(warning);
    }

    result.extend(input.into_token_stream());

    result.into()
}

struct BestBeforeArgs {
    warning_date: NaiveDate,
    expires_date: Option<NaiveDate>,
    message: Option<String>,
}

impl Parse for BestBeforeArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut warning_date = None;
        let mut expires_date = None;
        let mut message = None;
        
        if input.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                "Missing parameters. Expected either warning date or expires parameter",
            ));
        }
        
        if input.peek(LitStr) {
            let date_lit: LitStr = input.parse()?;
            warning_date = Some(parse_date(&date_lit.value()));
            
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        
        while !input.is_empty() {
            let name: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            
            if name == "expires" {
                let date_lit = input.parse::<LitStr>()?;
                expires_date = Some(parse_date(&date_lit.value()));
            } else if name == "message" {
                let msg_lit = input.parse::<LitStr>()?;
                message = Some(msg_lit.value());
            } else {
                return Err(syn::Error::new(
                    name.span(),
                    "Unknown parameter, expected 'expires' or 'message'",
                ));
            }
            
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        
        // If warning_date is not provided but expires_date is, use expires_date as the warning_date
        // This simplifies using #[bestbefore(expires="01.2028")] format
        if warning_date.is_none() {
            if let Some(exp_date) = expires_date {
                warning_date = Some(exp_date);
            } else {
                return Err(syn::Error::new(input.span(), 
                    "Missing parameters. You must provide either a warning date or an expires parameter"));
            }
        }
        
        Ok(BestBeforeArgs {
            warning_date: warning_date.unwrap(),
            expires_date,
            message,
        })
    }
}

fn parse_date(date_str: &str) -> NaiveDate {
    let parts: Vec<&str> = date_str.split('.').collect();
    if parts.len() != 2 {
        panic!(
            "Invalid date format: '{}'. Expected format: 'MM.YYYY'",
            date_str
        );
    }

    let month = parts[0].parse::<u32>().unwrap_or_else(|_| {
        panic!("Invalid month: '{}'. Expected a number from 1-12", parts[0]);
    });

    let year = parts[1].parse::<i32>().unwrap_or_else(|_| {
        panic!("Invalid year: '{}'. Expected a valid year number", parts[1]);
    });

    if month < 1 || month > 12 {
        panic!("Invalid month: {}. Expected a number from 1-12", month);
    }

    NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_else(|| {
        panic!("Invalid date: {}.{}", month, year);
    })
}

fn item_name(item: &syn::Item) -> String {
    match item {
        syn::Item::Fn(item_fn) => item_fn.sig.ident.to_string(),
        syn::Item::Mod(item_mod) => item_mod.ident.to_string(),
        syn::Item::Impl(_) => "implementation block".to_string(),
        syn::Item::Trait(item_trait) => format!("trait {}", item_trait.ident),
        syn::Item::Struct(item_struct) => format!("struct {}", item_struct.ident),
        syn::Item::Enum(item_enum) => format!("enum {}", item_enum.ident),
        _ => "code block".to_string(),
    }
}
