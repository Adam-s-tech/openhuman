use super::*;
use crate::runtime::python::bootstrap::PythonSource;

// `apply_no_window` is a no-op off Windows, but exercising the spawn path
// end-to-end keeps the GH-4814 CREATE_NO_WINDOW hook covered. `/bin/cat
// <file>` prints the file and exits, so it stands in for the python child.
