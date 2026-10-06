use crate::{SwapParam, SwapParamValue};

pub type StringParam = SwapParam<String>;
pub type StringParamValue = SwapParamValue<String>;

#[cfg(test)]
mod tests {
    use clack_plugin::utils::ClapId;

    use crate::Persistent;

    use super::*;

    #[test]
    fn default_and_set_value() {
        let param = StringParam::builder()
            .default(String::from("hello"))
            .build();

        assert_eq!(&*param.value(), "hello");
        assert_eq!(param.default_value(), "hello");

        param.set_value("world");
        assert_eq!(&*param.value(), "world");
        // Setting the value does not change the default.
        assert_eq!(param.default_value(), "hello");
    }

    #[test]
    fn default_is_empty_without_one() {
        let param = StringParam::builder().build();
        assert_eq!(&*param.value(), "");
    }

    #[test]
    fn persistent_roundtrip() {
        let param = StringParam::create(
            ClapId::new(7),
            String::from("pattern"),
            Some(String::from("seq")),
        );
        param.set_value("x..x..x.");

        let mut buf = Vec::new();
        param.serialize(&mut buf).unwrap();

        let restored = StringParam::create(ClapId::new(7), String::from("pattern"), None);
        restored.deserialize(&mut buf.as_slice()).unwrap();

        assert_eq!(&*restored.value(), "x..x..x.");
    }

    #[test]
    fn identity() {
        let param = StringParam::create(
            ClapId::new(42),
            String::from("sample"),
            Some(String::from("osc")),
        );

        assert_eq!(param.id(), ClapId::new(42));
        assert_eq!(param.name(), "sample");
        assert_eq!(param.module(), Some("osc"));
        assert_eq!(param.path(), "osc.sample");
    }

    #[test]
    fn deserialize_invalid_data() {
        let param = StringParam::builder().build();
        let mut reader = std::io::Cursor::new(b"not a string");
        assert!(param.deserialize(&mut reader).is_err());
    }
}
