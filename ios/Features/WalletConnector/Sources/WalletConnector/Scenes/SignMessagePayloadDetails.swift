// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemListRow
import Localization
import PrimitivesComponents
import SwiftUI

public struct SignMessagePayloadDetails: View {
    private let primaryRows: [GemListRow]
    private let secondaryRows: [GemListRow]
    private let messageText: String
    private let onSelectAddress: (String) -> Void

    public init(model: SignMessageSceneViewModel) {
        self.init(primaryRows: model.primaryPayloadFields, secondaryRows: model.secondaryPayloadFields, messageText: model.messageText, onSelectAddress: model.onSelectPayloadAddress)
    }

    public init(model: AuthenticationSceneViewModel) {
        self.init(primaryRows: model.primaryPayloadFields, secondaryRows: model.secondaryPayloadFields, messageText: model.messageText ?? "", onSelectAddress: model.onSelectPayloadAddress)
    }

    private init(
        primaryRows: [GemListRow],
        secondaryRows: [GemListRow],
        messageText: String,
        onSelectAddress: @escaping (String) -> Void,
    ) {
        self.primaryRows = primaryRows
        self.secondaryRows = secondaryRows
        self.messageText = messageText
        self.onSelectAddress = onSelectAddress
    }

    public var body: some View {
        if primaryRows.isNotEmpty || secondaryRows.isNotEmpty {
            NavigationStack {
                SimulationPayloadDetailsScene(
                    primaryRows: primaryRows,
                    secondaryRows: secondaryRows,
                    onSelectAddress: onSelectAddress,
                    actionListItem: ListItemModel(title: Localized.SignMessage.viewFullMessage),
                    actionDestination: AnyView(TextMessageScene(text: messageText)),
                )
            }
            .sheetPresentation([.large])
        }
    }
}
