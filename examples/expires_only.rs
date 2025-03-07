use bestbefore::bestbefore;

// This example demonstrates that we can use the macro with only the expires parameter,
// without specifying a warning date

#[bestbefore(expires = "01.2028")]
fn function_with_only_expiration_date() {
    println!("This function will cause a compilation error if compiled after January 2028");
}

#[bestbefore(expires = "01.2028", message = "This code must be removed by 2028")]
fn expiration_with_custom_message() {
    println!("This function will cause a compilation error with a custom message if compiled after January 2028");
}

fn main() {
    function_with_only_expiration_date();
    expiration_with_custom_message();
    println!("Example completed!");
}
