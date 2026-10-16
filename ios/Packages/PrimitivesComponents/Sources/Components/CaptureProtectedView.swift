// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

struct CaptureProtectedView<Content: View> {
    private let content: Content

    init(@ViewBuilder content: () -> Content) {
        self.content = content()
    }
}

extension CaptureProtectedView: UIViewRepresentable {
    func makeUIView(context _: Context) -> CaptureProtectedContainerView<Content> {
        CaptureProtectedContainerView(rootView: content)
    }

    func updateUIView(_ uiView: CaptureProtectedContainerView<Content>, context _: Context) {
        uiView.hostingController.rootView.content = content
    }

    func sizeThatFits(_ proposal: ProposedViewSize, uiView: CaptureProtectedContainerView<Content>, context _: Context) -> CGSize? {
        uiView.hostingController.sizeThatFits(in: proposal.replacingUnspecifiedDimensions(by: CGSize(width: CGFloat.infinity, height: .infinity)))
    }
}
