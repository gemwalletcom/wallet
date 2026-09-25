// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import PrimitivesComponents
import Style
import SwiftUI

public struct TransactionScene: View {
    private let model: TransactionSceneViewModel

    public init(model: TransactionSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        ListSectionView(sections: model.sections) { item in
            content(for: model.itemModel(for: item))
        }
        .contentMargins([.top], .small, for: .scrollContent)
        .listSectionSpacing(.compact)
        .background(Colors.grayBackground)
        .navigationTitle(model.title)
    }

    @ViewBuilder
    private func content(for itemModel: TransactionItemModel) -> some View {
        switch itemModel {
        case let .fee(model):
            NavigationCustomLink(
                with: ListItemView(model: model),
                action: self.model.onSelectFeeDetails,
            )
        case let .header(headerType):
            TransactionHeaderListItemView(
                headerType: headerType,
                action: model.onTransactionHeaderTap,
            )
        case let .swapProgress(model):
            TransactionSwapProgressView(model: model)
        case let .participant(model):
            AddressListItemView(model: model)
        case let .row(row):
            GemListRowView(row: row, onSelectAddress: model.onSelectProviderContract, onInfo: model.onInfo)
        case let .swapAgain(text):
            let button = StateButton(
                text: text,
                type: .primary(.normal),
                action: model.onSelectSwapAgain,
            )
            .cleanListRow(topOffset: .zero)
            if #available(iOS 26, *) {
                button.cornerRadius(.scene.button.height / 2)
            }
        case .empty:
            EmptyView()
        }
    }
}
