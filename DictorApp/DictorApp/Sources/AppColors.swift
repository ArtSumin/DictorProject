import SwiftUI

/// Семантические цвета из ассетов — акценты, действия, состояния.
enum AppColors {
    /// Основной акцент — активное состояние, primary actions, switch on
    static let accentPrimary = Color("AccentPrimary")
    /// Дополнительный акцент — градиенты, вторичные элементы
    static let accentSecondary = Color("AccentSecondary")
    /// Деструктивное действие — остановка, предупреждение
    static let accentDestructive = Color("AccentDestructive")
    /// Недоступный элемент — блокировка, disabled controls (сервер offline и т.п.)
    static let controlDisabled = Color("ControlDisabled")
}
