/// Types of coupling problems
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IssueType {
    /// Strong coupling spanning a long distance
    GlobalComplexity,
    /// Strong coupling to a frequently changing component
    CascadingChangeRisk,
    /// Intrusive coupling across boundaries (field/internals access)
    InappropriateIntimacy,
    /// A module with too many dependencies
    HighEfferentCoupling,
    /// A module that too many others depend on
    HighAfferentCoupling,
    /// Weak coupling where stronger might be appropriate
    UnnecessaryAbstraction,
    /// Circular dependency detected
    CircularDependency,
    /// Strong temporal co-change without an explicit code dependency
    HiddenCoupling,
    /// Supporting or generic module changing more often than expected
    AccidentalVolatility,
    /// Direct coupling to a third-party crate is spread across many modules
    ScatteredExternalCoupling,

    // === APOSD-inspired issues (A Philosophy of Software Design) ===
    /// Module with interface complexity close to implementation complexity
    ShallowModule,
    /// Method that only delegates to another method without adding value
    PassThroughMethod,
    /// Module requiring too much knowledge to understand/modify
    HighCognitiveLoad,

    // === Khononov/Rust-specific issues ===
    /// Module with too many functions, types, or implementations
    GodModule,
    /// Public fields exposed to external modules (should use getters/methods)
    PublicFieldExposure,
    /// Functions with too many primitive parameters (consider newtype)
    PrimitiveObsession,
}

impl std::fmt::Display for IssueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueType::GlobalComplexity => write!(f, "Global Complexity"),
            IssueType::CascadingChangeRisk => write!(f, "Cascading Change Risk"),
            IssueType::InappropriateIntimacy => write!(f, "Inappropriate Intimacy"),
            IssueType::HighEfferentCoupling => write!(f, "High Efferent Coupling"),
            IssueType::HighAfferentCoupling => write!(f, "High Afferent Coupling"),
            IssueType::UnnecessaryAbstraction => write!(f, "Unnecessary Abstraction"),
            IssueType::CircularDependency => write!(f, "Circular Dependency"),
            IssueType::HiddenCoupling => write!(f, "Hidden Coupling"),
            IssueType::AccidentalVolatility => write!(f, "Accidental Volatility"),
            IssueType::ScatteredExternalCoupling => write!(f, "Scattered External Coupling"),
            // APOSD-inspired
            IssueType::ShallowModule => write!(f, "Shallow Module"),
            IssueType::PassThroughMethod => write!(f, "Pass-Through Method"),
            IssueType::HighCognitiveLoad => write!(f, "High Cognitive Load"),
            // Khononov/Rust-specific
            IssueType::GodModule => write!(f, "God Module"),
            IssueType::PublicFieldExposure => write!(f, "Public Field Exposure"),
            IssueType::PrimitiveObsession => write!(f, "Primitive Obsession"),
        }
    }
}

impl IssueType {
    /// Get a detailed description of what this issue type means
    pub fn description(&self) -> &'static str {
        match self {
            IssueType::GlobalComplexity => {
                "Strong coupling to distant components increases cognitive load and makes the system harder to understand and modify."
            }
            IssueType::CascadingChangeRisk => {
                "Strongly coupling to volatile components means changes will cascade through the system, requiring updates in many places."
            }
            IssueType::InappropriateIntimacy => {
                "Direct access to internal details (fields, private methods) across module boundaries violates encapsulation."
            }
            IssueType::HighEfferentCoupling => {
                "A module depending on too many others is fragile and hard to test. Changes anywhere affect this module."
            }
            IssueType::HighAfferentCoupling => {
                "A module that many others depend on is hard to change. Any modification risks breaking dependents."
            }
            IssueType::UnnecessaryAbstraction => {
                "Using abstract interfaces for closely-related stable components may add complexity without benefit."
            }
            IssueType::CircularDependency => {
                "Circular dependencies make it impossible to understand, test, or modify components in isolation."
            }
            IssueType::HiddenCoupling => {
                "Files frequently change together without an explicit code dependency. This suggests implicit shared knowledge or a missing abstraction."
            }
            IssueType::AccidentalVolatility => {
                "A supporting or generic subdomain changes frequently despite being expected to be stable. This suggests churn from design or ownership issues rather than essential business volatility."
            }
            IssueType::ScatteredExternalCoupling => {
                "A third-party crate is used directly from many internal modules, spreading upgrade and API-change risk across code you control."
            }
            // APOSD-inspired descriptions
            IssueType::ShallowModule => {
                "Interface complexity is close to implementation complexity. The module doesn't hide enough complexity behind a simple interface. (APOSD: Deep vs Shallow Modules)"
            }
            IssueType::PassThroughMethod => {
                "Method only delegates to another method without adding significant functionality. Indicates unclear responsibility division. (APOSD: Pass-Through Methods)"
            }
            IssueType::HighCognitiveLoad => {
                "Module requires too much knowledge to understand and modify. Too many public APIs, dependencies, or complex type signatures. (APOSD: Cognitive Load)"
            }
            // Khononov/Rust-specific descriptions
            IssueType::GodModule => {
                "Module has too many responsibilities - too many functions, types, or implementations. Consider splitting into focused, cohesive modules. (SRP violation)"
            }
            IssueType::PublicFieldExposure => {
                "Struct has public fields accessed from other modules. Consider using getter methods to reduce coupling and allow future implementation changes."
            }
            IssueType::PrimitiveObsession => {
                "Function has many primitive parameters of the same type. Consider using newtype pattern (e.g., `struct UserId(u64)`) for type safety and clarity."
            }
        }
    }

    /// Get a Japanese description of what this issue type means.
    pub fn description_japanese(&self) -> &'static str {
        match self {
            IssueType::GlobalComplexity => {
                "遠いコンポーネントへの強い結合は認知負荷を高め、理解や変更を難しくします。"
            }
            IssueType::CascadingChangeRisk => {
                "頻繁に変わるコンポーネントへ強く結合すると、変更がシステム全体に波及しやすくなります。"
            }
            IssueType::InappropriateIntimacy => {
                "モジュール境界を越えた内部詳細への直接アクセスはカプセル化を損ないます。"
            }
            IssueType::HighEfferentCoupling => {
                "多くのモジュールに依存するモジュールは壊れやすく、テストも難しくなります。"
            }
            IssueType::HighAfferentCoupling => {
                "多くのモジュールから依存されるモジュールは変更しづらく、依存元を壊すリスクがあります。"
            }
            IssueType::UnnecessaryAbstraction => {
                "近く安定したコンポーネントに抽象インターフェースを使うと、利益より複雑さが増える場合があります。"
            }
            IssueType::CircularDependency => {
                "循環依存はコンポーネントを単独で理解、テスト、変更することを難しくします。"
            }
            IssueType::HiddenCoupling => {
                "明示的なコード依存がないのにファイルが頻繁に一緒に変わっています。暗黙の知識や不足した抽象化を示している可能性があります。"
            }
            IssueType::AccidentalVolatility => {
                "安定しているはずの支援/汎用サブドメインが頻繁に変更されています。設計や所有権の問題によるチャーンの可能性があります。"
            }
            IssueType::ScatteredExternalCoupling => {
                "サードパーティクレートが多くの内部モジュールから直接使われており、更新やAPI変更のリスクが広がっています。"
            }
            IssueType::ShallowModule => {
                "インターフェースの複雑さが実装の複雑さに近く、単純なインターフェースの背後に十分な複雑さを隠せていません。"
            }
            IssueType::PassThroughMethod => {
                "メソッドが価値を追加せず別メソッドへ委譲しており、責務分担が曖昧な可能性があります。"
            }
            IssueType::HighCognitiveLoad => {
                "公開API、依存、複雑な型シグネチャが多く、理解や変更に必要な知識が多すぎます。"
            }
            IssueType::GodModule => {
                "関数、型、実装が多すぎて責務が集中しています。焦点の絞られたモジュールへの分割を検討してください。"
            }
            IssueType::PublicFieldExposure => {
                "構造体の公開フィールドが他モジュールから使われています。getterなどで結合を弱めることを検討してください。"
            }
            IssueType::PrimitiveObsession => {
                "同じプリミティブ型の引数が多すぎます。newtypeパターンで型安全性と明確さを高めることを検討してください。"
            }
        }
    }
}
