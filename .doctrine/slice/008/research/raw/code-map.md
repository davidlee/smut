# Thread 2 — code map (raw)

## Hotspots (grep -n)
```
36:pub struct Set {
51:pub const HYGIENE: Set = Set {
59:pub const SETS: [&Set; 1] = [&HYGIENE];
66:pub enum Ask {
80:pub fn resolve(name: Option<&str>, format: Option<&str>) -> Result<Ask, Reject> {
100:pub fn answer(ask: Ask) -> String {
110:pub fn listing() -> String {
124:pub fn seed(set: &Set) -> Value {
141:fn id_of(set: &Set) -> String {
147:fn title_of(set: &Set) -> String {
167:fn body_of(set: &Set) -> String {
172:fn set_named(found: &str) -> Result<&'static Set, Reject> {
183:fn needs_a_name() -> String {
45:    Conventions(conventions::Ask),
286:    Conventions {
314:        Sub::Conventions { name, format } => Ok(Invocation::Conventions(conventions::resolve(
93:        Invocation::Conventions(ask) => Ok(conventions::answer(ask)),
94:        Invocation::Request(request) => request.run(&Store::in_directory(current_directory()?)),
100:fn current_directory() -> Result<PathBuf, StoreError> {
102:        step: "current_directory",
144:    UnknownConvention {
151:    UnknownConventionFormat {
372:            Reject::UnknownConvention { found, sets } => write!(
377:            Reject::UnknownConventionFormat { format } => write!(
657:                Reject::UnknownConvention {
665:                Reject::UnknownConventionFormat {
263:    pub fn exit_code(&self) -> u8 {
264-        match self {
265-            Error::Reject(_) => INVALID_INPUT,
266-            Error::Store(_) => STORE_FAILURE,
267-        }
268-    }
269-}
```

## Greps that returned nothing
```
$ grep -rn "\.git\b" --include=*.rs src/   # no git-root detection exists
$ grep -rn "home_dir\|XDG" --include=*.rs src/   # no config-home reading exists
```

## Test helpers inherit the environment
```
fn run(scratch: &Scratch, arguments: &[&str], stdin: Option<&str>) -> Ran {
    let mut child = Command::new(env!("CARGO_BIN_EXE_smut"))
        .current_dir(scratch.path())
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary starts");

    if let Some(text) = stdin {
        child
```
