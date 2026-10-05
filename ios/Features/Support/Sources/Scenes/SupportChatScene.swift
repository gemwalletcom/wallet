// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemSupportChatGroup
import Localization
import PrimitivesComponents
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
                        SupportDateSeparator(title: day.title)
                        ForEach(day.groups, id: \.rows.first?.message.id) { group in
                            groupView(group)
                        }
                    }
                    if model.typingAgentName != nil {
                        SupportTypingIndicator()
                            .transition(.opacity)
                    }
                }
                .padding(.medium)
                .animation(.smooth, value: model.typingAgentName)
            }
            .defaultScrollAnchor(.bottom)
            switch model.phase {
            case .rows: EmptyView()
            case let .error(error):
                ListItemErrorView(errorTitle: Localized.Errors.errorOccurred, error: error)
                    .padding(.medium)
            case let .empty(state):
                EmptyContentView(model: EmptyStateViewModel(state: state))
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
        .toolbar {
            ToolbarItem(placement: .principal) {
                HStack(spacing: .small) {
                    AssetImageView(
                        assetImage: AssetImage(imageURL: nil, placeholder: Images.Support.agent),
                        size: .image.small,
                    )
                    Text(model.title)
                        .font(.headline)
                }
            }
        }
        .interactiveDismissDisabled()
        .task {
            await model.load()
        }
        .taskOnce {
            Task { await model.enableNotificationsForSupport() }
        }
        .onChange(of: scenePhase, model.onScenePhaseChange)
        .onDisappear { model.onDisappear() }
        .quickLookPreview($model.previewURL)
        .alertSheet($model.isPresentingAlertMessage)
    }

    @ViewBuilder
    private func groupView(_ group: GemSupportChatGroup) -> some View {
        switch group.side {
        case .incoming:
            SupportAgentMessageGroup(rows: group.rows, onRetry: model.onRetry, onImage: model.onOpenPreview)
        case .outgoing:
            SupportUserMessageGroup(rows: group.rows, onRetry: model.onRetry, onImage: model.onOpenPreview)
        }
    }
}
