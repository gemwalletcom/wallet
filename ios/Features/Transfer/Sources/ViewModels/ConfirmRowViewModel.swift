// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemConfirmDestination
import enum Gemstone.GemConfirmRowContent
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents

struct ConfirmRowViewModel {
    private let content: GemConfirmRowContent
    private let onSelectAddress: (@MainActor @Sendable (ChainAddress) -> Void)?

    init(
        content: GemConfirmRowContent,
        onSelectAddress: (@MainActor @Sendable (ChainAddress) -> Void)? = nil,
    ) {
        self.content = content
        self.onSelectAddress = onSelectAddress
    }
}

// MARK: - ItemModelProvidable

extension ConfirmRowViewModel: ItemModelProvidable {
    var itemModel: ConfirmTransferItemModel {
        switch content {
        case let .row(row):
            .row(row)
        case let .recipient(destination, name, text, address, memo, chain, link, avatar, isSelectable):
            .recipient(
                AddressListItemViewModel(
                    title: destination.title,
                    account: SimpleAccount(
                        name: name,
                        chain: Chain(core: chain),
                        address: address,
                        memo: memo,
                        assetImage: avatar.map(\.assetImage),
                    ),
                    mode: .text(text),
                    addressLink: link.toPrimitives(),
                    onSelect: isSelectable ? selectAction(chainAddress: ChainAddress(chain: Chain(core: chain), address: address)) : nil,
                ),
            )
        case let .paymentAsset(symbol, selectable, _):
            .paymentAsset(ListItemModel(title: Localized.Transfer.payWith, subtitle: symbol), selectable: selectable)
        case .details:
            .empty
        }
    }
}

// MARK: - Private

extension ConfirmRowViewModel {
    private func selectAction(chainAddress: ChainAddress) -> (@MainActor @Sendable () -> Void)? {
        guard let onSelectAddress else { return nil }
        return { onSelectAddress(chainAddress) }
    }
}

extension GemConfirmRowContent {
    func item(at index: Int) -> ConfirmTransferItem {
        switch self {
        case .row, .recipient, .paymentAsset: .row(index)
        case .details: .details
        }
    }
}
