// Derived values only. Every user input is declared in cargo-generate.toml, so generation
// works non-interactively (--silent) and never runs system commands.
let project = variable::get("project-name");          // e.g. acme-svc   (crate_name = acme_svc is built in)
variable::set("env_prefix", to_shouty_snake_case(project)); // ACME_SVC

if !variable::is_set("year") {
    variable::set("year", `${system::date().year}`);
}

let license = variable::get("license");
if license == "MIT" {
    file::delete("LICENSE-APACHE");
    file::rename("LICENSE-MIT", "LICENSE");
} else if license == "Apache-2.0" {
    file::delete("LICENSE-MIT");
    file::rename("LICENSE-APACHE", "LICENSE");
} else {
    // "None": no LICENSE file and no `license` field in Cargo.toml (private project).
    file::delete("LICENSE-MIT");
    file::delete("LICENSE-APACHE");
}

file::delete("hooks");
