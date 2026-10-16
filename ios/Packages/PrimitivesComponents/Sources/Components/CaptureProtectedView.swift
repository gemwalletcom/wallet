// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

public struct CaptureProtectedView<Content: View> {
    private let content: Content

    public init(@ViewBuilder content: () -> Content) {
        self.content = content()
    }
}

extension CaptureProtectedView: UIViewRepresentable {
    public func makeCoordinator() -> UIHostingController<Content> {
        let hostingController = UIHostingController(rootView: content)
        hostingController.safeAreaRegions = []
        hostingController.view.backgroundColor = .clear
        return hostingController
    }

    public func makeUIView(context: Context) -> UIView {
        let container = Self.secureCanvas() ?? UIView()
        let hostingView: UIView = context.coordinator.view
        hostingView.frame = container.bounds
        hostingView.autoresizingMask = [.flexibleWidth, .flexibleHeight]
        container.addSubview(hostingView)
        return container
    }

    public func updateUIView(_: UIView, context: Context) {
        context.coordinator.rootView = content
    }

    public func sizeThatFits(_ proposal: ProposedViewSize, uiView _: UIView, context: Context) -> CGSize? {
        context.coordinator.sizeThatFits(in: proposal.replacingUnspecifiedDimensions(by: CGSize(width: CGFloat.infinity, height: .infinity)))
    }

    @MainActor
    private static func secureCanvas() -> UIView? {
        let field = UITextField()
        field.isSecureTextEntry = true
        field.layoutIfNeeded()
        guard let canvas = field.subviews.first(where: { String(describing: type(of: $0)).hasSuffix("CanvasView") }) else {
            return nil
        }
        canvas.subviews.forEach { $0.removeFromSuperview() }
        canvas.removeFromSuperview()
        return canvas
    }
}
