use ssbh_data::anim_data::GroupType;
use ssbh_data::matl_data::{
    BlendFactor, CullMode, FillMode, MagFilter, MaxAnisotropy, MinFilter, ParamId, WrapMode,
};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use strum::VariantNames;

fn write_enum_pymethods<W: Write>(w: &mut W, class_name: &str, enum_path: &str, variants: &[&str]) {
    writeln!(w, "#[pymethods]").unwrap();
    writeln!(w, "impl {} {{", class_name).unwrap();
    for variant in variants {
        writeln!(w, "    #[classattr]").unwrap();
        writeln!(w, "    #[pyo3(name = {:?})]", variant).unwrap();
        writeln!(
            w,
            "    pub fn {}() -> {} {{",
            variant.to_lowercase(),
            class_name
        )
        .unwrap();
        writeln!(
            w,
            "        {}::{}::{}.into()",
            enum_path, class_name, variant
        )
        .unwrap();
        writeln!(w, "    }}").unwrap();
        writeln!(w).unwrap();
    }

    // TODO: This would be cleaner as part of a macro?
    // TODO: Multiple pymethods blocks?
    // usize -> enum
    writeln!(w, "    #[staticmethod]").unwrap();
    writeln!(
        w,
        "    pub fn from_value(value: usize) -> Option<{}> {{",
        class_name
    )
    .unwrap();
    writeln!(
        w,
        "        {}::{}::from_repr(value).map(Into::into)",
        enum_path, class_name
    )
    .unwrap();
    writeln!(w, "    }}").unwrap();

    writeln!(w).unwrap();

    // String -> enum
    writeln!(w, "    #[staticmethod]").unwrap();
    writeln!(
        w,
        "    pub fn from_str(value: &str) -> Option<{}> {{",
        class_name
    )
    .unwrap();
    writeln!(
        w,
        "        {}::{}::from_str(value).map(Into::into).ok()",
        enum_path, class_name
    )
    .unwrap();
    writeln!(w, "    }}").unwrap();
    writeln!(w, "}}").unwrap();
}

fn main() {
    // TODO: When will this be rerun?
    // println!("cargo:rerun-if-changed=src/matl_data.rs");

    // TODO: Combine this with defining the enum class itself?
    generate_enum_file(
        "src/matl_data/enums.rs",
        "ssbh_data::matl_data",
        &[
            ("ParamId", ParamId::VARIANTS),
            ("BlendFactor", BlendFactor::VARIANTS),
            ("FillMode", FillMode::VARIANTS),
            ("CullMode", CullMode::VARIANTS),
            ("WrapMode", WrapMode::VARIANTS),
            ("MinFilter", MinFilter::VARIANTS),
            ("MagFilter", MagFilter::VARIANTS),
            ("MaxAnisotropy", MaxAnisotropy::VARIANTS),
        ],
    );

    generate_enum_file(
        "src/anim_data/enums.rs",
        "ssbh_data::anim_data",
        &[("GroupType", GroupType::VARIANTS)],
    );
}

fn generate_enum_file(file_path: &str, enum_path: &str, enums: &[(&str, &[&str])]) {
    // Make sure the folder exists first.
    let file_path = Path::new(file_path);
    std::fs::create_dir_all(file_path.parent().unwrap()).unwrap();

    let mut f = BufWriter::new(File::create(file_path).unwrap());
    writeln!(&mut f, "// File automatically generated by build.rs.").unwrap();
    writeln!(&mut f, "// Changes made to this file will not be saved.").unwrap();
    writeln!(&mut f, "use pyo3::prelude::*;").unwrap();
    writeln!(&mut f, "use super::*;").unwrap();
    writeln!(&mut f, "use std::str::FromStr;").unwrap();
    writeln!(&mut f).unwrap();
    for (name, variants) in enums {
        write_enum_pymethods(&mut f, name, enum_path, variants);

        // Each enum uses the same class structure for now.
        writeln!(&mut f, "impl crate::PyiClass for {} {{", name).unwrap();

        writeln!(&mut f, "    fn pyi_class() -> String {{").unwrap();
        writeln!(
            &mut f,
            r#"        "class {}:\n    name: str\n    value: int".to_string()"#,
            name
        )
        .unwrap();
        writeln!(&mut f, "    }}").unwrap();

        writeln!(&mut f, "}}").unwrap();

        // Add a class variable for each enum variant.
        // TODO: Is there a way to differentiate between class and instance variables?
        // HACK: Just use the methods trait to also optionally include class attributes.
        writeln!(&mut f, "impl crate::PyiMethods for {} {{", name).unwrap();
        writeln!(&mut f, "    fn pyi_methods() -> String {{").unwrap();

        let class_attributes = variants
            .iter()
            .map(|v| format!("    {}: ClassVar[{}]", v, name))
            .collect::<Vec<String>>()
            .join("\n");
        writeln!(
            &mut f,
            r#"        "{}

    @staticmethod
    def from_value(value: int) -> Optional[{}]: ...

    @staticmethod
    def from_str(value: str) -> Optional[{}]: ...".to_string()"#,
            class_attributes, name, name
        )
        .unwrap();

        writeln!(&mut f, "    }}").unwrap();
        writeln!(&mut f, "}}").unwrap();

        // We don't allow constructing enums directly, so just pick the appropriate class attribute.
        writeln!(&mut f, "impl crate::PyRepr for {} {{", name).unwrap();
        writeln!(&mut f, "    fn py_repr(&self) -> String {{").unwrap();
        writeln!(
            &mut f,
            "        \"{}.{}\".to_string()",
            enum_path.replace("ssbh_data::", "ssbh_data_py."),
            name
        )
        .unwrap();
        writeln!(&mut f, "    }}").unwrap();
        writeln!(&mut f, "}}").unwrap();
        writeln!(&mut f).unwrap();
        writeln!(&mut f, "#[pyproto]").unwrap();
        writeln!(&mut f, "impl pyo3::PyObjectProtocol for {} {{", name).unwrap();
        writeln!(&mut f, "    fn __repr__(&self) -> String {{").unwrap();
        writeln!(&mut f, "        self.py_repr()").unwrap();
        writeln!(&mut f, "    }}").unwrap();
        writeln!(&mut f, "}}").unwrap();
    }
}
