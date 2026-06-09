// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import QuickLook
import Store
import Style
import SwiftUI

public struct SupportChatScene: View {
    @State private var model: SupportChatSceneViewModel

    public init(model: SupportChatSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        content
            .bindQuery(model.query)
            .background(Colors.grayBackground)
            .safeAreaView(edge: .bottom) {
                SupportMessageInputBar(model: model.inputBarModel)
            }
            .navigationTitle(model.title)
            .navigationBarTitleDisplayMode(.inline)
            .task {
                await model.fetch()
            }
            .quickLookPreview($model.isPresentingImagePreview, in: model.previewURLs)
    }

    @ViewBuilder
    private var content: some View {
        ZStack {
            scrollContent
            if model.isEmpty {
                StateEmptyView(
                    title: model.emptyTitle,
                    description: model.emptyDescription,
                    image: Image(systemName: SystemImage.bubbleLeftAndBubbleRight),
                )
                .padding(.medium)
            }
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
