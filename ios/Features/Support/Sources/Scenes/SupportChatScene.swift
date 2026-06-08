// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Store
import Style
import SwiftUI

public struct SupportChatScene: View {
    @State private var model: SupportChatSceneViewModel

    public init(model: SupportChatSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        scrollContent
            .bindQuery(model.query)
            .background(Colors.grayBackground)
            .overlay {
                if model.isEmpty {
                    StateEmptyView(
                        title: model.emptyTitle,
                        description: model.emptyDescription,
                        image: Image(systemName: SystemImage.bubbleLeftAndBubbleRight),
                    )
                    .padding(.medium)
                }
            }
            .safeAreaView(edge: .bottom) {
                SupportMessageInputBar(model: model.inputBarModel)
            }
            .navigationTitle(model.title)
            .navigationBarTitleDisplayMode(.inline)
            .task {
                await model.fetch()
            }
    }

    @ViewBuilder
    private var scrollContent: some View {
        if #available(iOS 18, *) {
            SupportChatMessagesView(model: model)
        } else {
            ScrollView {
                SupportChatMessagesContent(model: model)
            }
            .defaultScrollAnchor(.bottom)
        }
    }
}
