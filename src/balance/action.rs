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
                    "Introduce trait `{}` with methods: {}",
                    suggested_name,
                    methods.join(", ")
                )
            }
            RefactoringAction::MoveCloser { target_location } => {
                write!(f, "Move component to `{}`", target_location)
            }
            RefactoringAction::ExtractAdapter {
                adapter_name,
                purpose,
            } => {
                write!(f, "Extract adapter `{}` to {}", adapter_name, purpose)
            }
            RefactoringAction::SplitModule { suggested_modules } => {
                write!(f, "Split into modules: {}", suggested_modules.join(", "))
            }
            RefactoringAction::SimplifyAbstraction { direct_usage } => {
                write!(f, "Replace with direct usage: {}", direct_usage)
            }
            RefactoringAction::BreakCycle {
                suggested_direction,
            } => {
                write!(f, "Break cycle by {}", suggested_direction)
            }
            RefactoringAction::StabilizeInterface { interface_name } => {
                write!(f, "Add stable interface `{}`", interface_name)
            }
            RefactoringAction::General { action } => {
                write!(f, "{}", action)
            }
            RefactoringAction::AddGetters { fields } => {
                write!(f, "Add getter methods for: {}", fields.join(", "))
            }
            RefactoringAction::IntroduceNewtype {
                suggested_name,
                wrapped_type,
            } => {
                write!(
                    f,
                    "Introduce newtype: `struct {}({});`",
                    suggested_name, wrapped_type
                )
            }
        }
    }
}
