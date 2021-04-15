Updating the Rust code examples
-------------------------------

* Finish going through the existing examples,
  updating them from the TEMPLATE.rs and TEMPLATE.toml files.

* Copy the src/main.rs files from my repo to the smithy repo,
  in the appropriate branch.

* Make sure the Cargo.toml files, in the smithy repo's examples directory, have unique names.

* Make sure the Cargo.toml files also have:
  aws-types = { path = "../../build/aws-sdk/aws-types" }

