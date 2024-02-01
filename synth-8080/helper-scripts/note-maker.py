import pandas as pd



def mk_var_name(name):
    return name.replace("#", "s")


def mk_variant(name):
    return f"\t#[serde(alias = \"{name.lower()}\", alias = \"{name}\")]\n\t{mk_var_name(name)},"


def mk_enum(names, notes):
    body = "\n".join([mk_variant(name) for name in names])
    return f"#[derive(Debug, Clone, Copy, Deserialize)]\npub enum Note {{\n{body}\n}}"


def mk_impl_float(names, notes):
    head = "impl Into<Float> for Note {"
    tail = "\t}\n}"
    f_decleration = "\tfn into(self) -> Float {"
    match_body = "\n".join([f"\t\t\tNote::{mk_var_name(name)} => {notes.get(name)}," for name in names])

    return "\n".join([head, f_decleration, "\t\tmatch self {", match_body, "\t\t}", tail])


def mk_impl_display(names):
    head = "impl Display for Note {"
    tail = "\t}\n}"
    f_decleration = "\tfn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {"
    match_body = "\n".join([f"\t\t\tNote::{mk_var_name(name)} => write!(f, \"{name}\")," for name in names])

    return "\n".join([head, f_decleration, "\t\tmatch self {", match_body, "\t\t}", tail])


def main():
    df = pd.read_html('https://pages.mtu.edu/~suits/notefreqs.html')[1]

    notes = { name: row[1] for index, row in df.iterrows() for name in row[0].split('/')}
    names = [name for df_name in df[0] for name in df_name.split("/")]
    # print(notes)
    print("use crate::Float;")
    print("use serde::Deserialize;")
    print("use std::convert::Into;")
    print("use std::fmt::{Display, Formatter, self};")
    print()
    print(mk_enum(names, notes))
    print()
    print(mk_impl_float(names, notes))
    print()
    print(mk_impl_display(names))


if __name__ == "__main__":
   main() 
