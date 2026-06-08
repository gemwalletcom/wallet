// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

@available(iOS 18.0, *)
struct SupportChatMessagesView: View {
    let model: SupportChatSceneViewModel
    @State private var scrollPosition = ScrollPosition(idType: String.self)

    var body: some View {
        ScrollView {
            SupportChatMessagesContent(model: model)
        }
        .scrollPosition($scrollPosition)
        .defaultScrollAnchor(.bottom, for: .initialOffset)
        .defaultScrollAnchor(.bottom, for: .sizeChanges)
        .defaultScrollAnchor(.top, for: .alignment)
        .onScrollGeometryChange(for: CGFloat.self) { $0.contentInsets.bottom } action: { old, new in
            if new > old, model.isAtBottom {
                scrollPosition.scrollTo(edge: .bottom)
            }
        }
        .overlay(alignment: .bottomTrailing) {
            if !model.isAtBottom {
                SupportScrollToBottomButton(unreadCount: model.unreadAgentCount) {
                    model.setAtBottom(true)
                    withAnimation { scrollPosition.scrollTo(edge: .bottom) }
                }
                .padding(.medium)
            }
        }
    }
}
