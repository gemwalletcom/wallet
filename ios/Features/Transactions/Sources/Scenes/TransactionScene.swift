// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemTransactionDetailRow
import PrimitivesComponents
import Style
import SwiftUI

public struct TransactionScene: View {
    private let model: TransactionSceneViewModel

    public init(model: TransactionSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        ListSectionView(sections: model.sections) { row in
            content(for: row)
        }
        .contentMargins([.top], .small, for: .scrollContent)
        .listSectionSpacing(.compact)
        .background(Colors.grayBackground)
        .navigationTitle(model.title)
    }

    @ViewBuilder
    private func content(for row: GemTransactionDetailRow) -> some View {
        switch row {
        case let .fee(row):
            NavigationCustomLink(
                with: GemListRowView(row: row, onInfo: model.onInfo),
                action: model.onSelectFeeDetails,
            )
        case let .header(header):
            TransactionHeaderListItemView(
                header: header,
                action: model.onTransactionHeaderTap,
            )
        case let .swapProgress(progress):
            TransactionSwapProgressView(progress: progress)
        case let .participant(row):
            AddressListItemView(row: row, onSelect: model.selectAction(row), onAddContact: model.addContactAction)
        case let .row(row):
            GemListRowView(row: row, onSelectAddress: model.onSelectProviderContract, onInfo: model.onInfo)
        case let .swapAgain(title, swap):
            let button = StateButton(
                text: title.text,
                type: .primary(.normal),
                action: { model.onSelectSwapAgain(swap) },
            )
            .cleanListRow(topOffset: .zero)
            if #available(iOS 26, *) {
                button.cornerRadius(.scene.button.height / 2)
            }
        }
    }
}
