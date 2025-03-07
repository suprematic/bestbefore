use bestbefore::bestbefore;

// This will generate a warning if compiled after March 2024
#[bestbefore("03.2024")]
fn future_warning() {
    println!("This function will have a warning after March 2024");
}

// This will generate a warning if compiled after January 2023
// and fail to compile if after December 2023
#[bestbefore("01.2026", expires = "12.2030")]
fn expired_function() {
    println!("This function should fail to compile if after December 2030");
}

// This will generate a warning with a custom message if compiled after February 2023
#[bestbefore("02.2023", message = "Please use new_api() instead")]
fn deprecated_with_message() {
    println!("This function has a custom warning message");
}

// Example of applying to non-function items
#[bestbefore("06.2023")]
mod legacy_module {
    pub fn old_function() {
        println!("This entire module is marked for warning");
    }
}

#[bestbefore("01.2023", /*expires = "05.2023"*/)]
struct OldStructure {
    field: String,
}

fn main() {
    future_warning();
    expired_function();
    deprecated_with_message();
    legacy_module::old_function();

    let old = OldStructure {
        field: "test".to_string(),
    };
    println!("Old structure field: {}", old.field);
}
