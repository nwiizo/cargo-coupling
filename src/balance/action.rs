/// Specific refactoring actions
#[derive(Debug, Clone)]
pub enum RefactoringAction {
    /// Introduce a trait to abstract the coupling
    IntroduceTrait {
        suggested_name: String,
        methods: Vec<String>,
    },
    /// Move the component closer (same module/crate)
    MoveCloser { target_location: String },
    /// Extract an interface/adapter
    ExtractAdapter {
        adapter_name: String,
        purpose: String,
    },
    /// Split a large module
    SplitModule { suggested_modules: Vec<String> },
    /// Remove unnecessary abstraction
    SimplifyAbstraction { direct_usage: String },
    /// Break circular dependency
    BreakCycle { suggested_direction: String },
    /// Add stable interface
    StabilizeInterface { interface_name: String },
    /// General refactoring suggestion
    General { action: String },
    /// Add getter methods to replace direct field access
    AddGetters { fields: Vec<String> },
    /// Introduce newtype pattern for type safety
    IntroduceNewtype {
        suggested_name: String,
        wrapped_type: String,
    },
}

impl std::fmt::Display for RefactoringAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RefactoringAction::IntroduceTrait {
                suggested_name,
                methods,
            } => {
                write!(
                    f,
                    "If it hides volatile knowledge, consider trait `{}` with methods: {}",
                    suggested_name,
                    methods.join(", ")
                )
            }
            RefactoringAction::MoveCloser { target_location } => {
                write!(
                    f,
                    "If the responsibilities change together, consider moving the component to `{}`",
                    target_location
                )
            }
            RefactoringAction::ExtractAdapter {
                adapter_name,
                purpose,
            } => {
                write!(
                    f,
                    "If translation isolates volatile knowledge, consider adapter `{}` to {}",
                    adapter_name, purpose
                )
            }
            RefactoringAction::SplitModule { suggested_modules } => {
                write!(
                    f,
                    "If responsibilities evolve independently, consider boundaries such as: {}",
                    suggested_modules.join(", ")
                )
            }
            RefactoringAction::SimplifyAbstraction { direct_usage } => {
                write!(
                    f,
                    "If the boundary adds no needed protection, consider direct usage: {}",
                    direct_usage
                )
            }
            RefactoringAction::BreakCycle {
                suggested_direction,
            } => {
                write!(
                    f,
                    "If the cycle obstructs independent changes, consider {}",
                    suggested_direction
                )
            }
            RefactoringAction::StabilizeInterface { interface_name } => {
                write!(
                    f,
                    "If it hides unstable details, consider interface `{}`",
                    interface_name
                )
            }
            RefactoringAction::General { action } => {
                write!(f, "{}", action)
            }
            RefactoringAction::AddGetters { fields } => {
                write!(
                    f,
                    "Review whether these fields expose unintended knowledge before adding accessors: {}",
                    fields.join(", ")
                )
            }
            RefactoringAction::IntroduceNewtype {
                suggested_name,
                wrapped_type,
            } => {
                write!(
                    f,
                    "If it enforces a domain invariant, consider newtype: `struct {}({});`",
                    suggested_name, wrapped_type
                )
            }
        }
    }
}
