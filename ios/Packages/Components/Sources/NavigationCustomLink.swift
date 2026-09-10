import Style
import SwiftUI

public struct NavigationCustomLink<Content: View>: View {
    private let content: Content
    private let isEnabled: Bool
    private let action: @MainActor @Sendable () -> Void

    public init(
        with content: Content,
        isEnabled: Bool = true,
        action: @escaping @MainActor @Sendable () -> Void,
    ) {
        self.content = content
        self.isEnabled = isEnabled
        self.action = action
    }

    public var body: some View {
        Button(action: action) {
            HStack {
                content
                    .layoutPriority(1)
                if isEnabled {
                    NavigationLink.empty
                }
            }
        }
        .tint(Colors.black)
        .allowsHitTesting(isEnabled)
    }
}
