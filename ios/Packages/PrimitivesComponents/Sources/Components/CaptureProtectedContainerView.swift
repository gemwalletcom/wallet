// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

final class CaptureProtectedContainerView<Content: View>: UIView {
    let hostingController: UIHostingController<CaptureProtectedContent<Content>>
    private let secureField: UITextField

    init(rootView: Content) {
        let secureField = UITextField()
        let canvas = Self.secureCanvas(of: secureField)
        self.secureField = secureField
        hostingController = UIHostingController(rootView: CaptureProtectedContent(content: rootView, isProtected: canvas != nil))
        super.init(frame: .zero)

        hostingController.safeAreaRegions = []
        hostingController.view.backgroundColor = .clear

        if let canvas {
            fill(self, with: canvas)
            fill(canvas, with: hostingController.view)
        } else {
            fill(self, with: hostingController.view)
        }
    }

    @available(*, unavailable)
    required init?(coder _: NSCoder) {
        fatalError()
    }

    private static func secureCanvas(of field: UITextField) -> UIView? {
        field.isSecureTextEntry = true
        field.layoutIfNeeded()
        guard let canvas = field.subviews.first else {
            return nil
        }
        canvas.subviews.forEach { $0.removeFromSuperview() }
        canvas.removeFromSuperview()
        return canvas
    }

    private func fill(_ container: UIView, with view: UIView) {
        view.frame = container.bounds
        view.autoresizingMask = [.flexibleWidth, .flexibleHeight]
        container.addSubview(view)
    }
}
