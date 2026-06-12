// Copyright (c). Gem Wallet. All rights reserved.

import Components
import QuickLook
import Store
import Style
import SwiftUI

public struct SupportChatScene: View {
    @State private var model: SupportChatSceneViewModel
    @Environment(\.scenePhase) private var scenePhase

    public init(model: SupportChatSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        ZStack {
            ScrollView {
                VStack(spacing: .small) {
                    ForEach(model.days) { day in
                        SupportDateSeparator(date: day.date)
                        ForEach(day.groups) { group in
                            groupView(group)
                        }
                    }
                }
                .padding(.medium)
            }
            .defaultScrollAnchor(.bottom)
            if model.isEmpty {
                StateEmptyView(
                    title: model.emptyTitle,
                    description: model.emptyDescription,
                    image: Image(systemName: SystemImage.bubbleLeftAndBubbleRight),
                )
                .padding(.medium)
            }
        }
        .bindQuery(model.query)
        .background(Colors.grayBackground.ignoresSafeArea())
        .safeAreaView(edge: .bottom) {
            SupportMessageInputBar(model: model.inputBarModel)
        }
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .interactiveDismissDisabled()
        .task {
            await model.fetch()
        }
        .onChange(of: scenePhase, model.onScenePhaseChange)
        .quickLookPreview($model.previewURL)
    }

    @ViewBuilder
    private func groupView(_ group: SupportChatGroup) -> some View {
        switch group.kind {
        case let .agent(name, messages):
            SupportAgentMessageGroup(name: name, messages: messages)
        case let .user(messages):
            SupportUserMessageGroup(messages: messages)
        }
    }
}
