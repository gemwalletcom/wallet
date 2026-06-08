// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

struct SupportChatMessagesContent: View {
    let model: SupportChatSceneViewModel

    var body: some View {
        VStack(spacing: .small) {
            ForEach(model.days) { day in
                SupportDateSeparator(date: day.date)
                ForEach(day.groups) { group in
                    groupView(group)
                        .onVisibilityChange(active: group.isLast) { model.setAtBottom($0) }
                }
            }
        }
        .padding(.medium)
    }

    @ViewBuilder
    private func groupView(_ group: SupportChatGroup) -> some View {
        switch group.kind {
        case let .agent(header, messages):
            SupportAgentMessageGroup(header: header, messages: messages)
        case let .user(messages):
            SupportUserMessageGroup(messages: messages)
        }
    }
}
