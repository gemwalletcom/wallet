import SwiftUI

@available(iOS 26.0, *)
public struct LiquidGlassModifier: ViewModifier {
    private let tint: Color?
    private let interactive: Bool
    private let shape: AnyShape

    public init(tint: Color?, interactive: Bool, shape: AnyShape) {
        self.tint = tint
        self.interactive = interactive
        self.shape = shape
    }

    public func body(content: Content) -> some View {
        content
            .glassEffect(.regular.tint(tint).interactive(interactive), in: shape)
    }
}

public extension View {
    @ViewBuilder
    func liquidGlass<GlassShape: Shape>(
        tint: Color? = nil,
        interactive: Bool = true,
        in shape: GlassShape = Capsule(),
        fallback: (Self) -> some View,
    ) -> some View {
        if #available(iOS 26.0, *) {
            modifier(LiquidGlassModifier(tint: tint, interactive: interactive, shape: AnyShape(shape)))
        } else {
            fallback(self)
        }
    }

    func liquidGlass<GlassShape: Shape>(
        tint: Color? = nil,
        interactive: Bool = true,
        in shape: GlassShape = Capsule(),
    ) -> some View {
        liquidGlass(
            tint: tint,
            interactive: interactive,
            in: shape,
            fallback: { $0 },
        )
    }
}
