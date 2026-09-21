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
        case let .recipient(destination, addressName, memo, chain, link):
            .recipient(
                recipientItem(
                    destination: destination,
                    addressName: addressName?.toPrimitives(),
                    memo: memo,
                    chain: Chain(core: chain),
                    link: link.toPrimitives(),
                ),
            )
        case let .paymentAsset(symbol, selectable):
            .paymentAsset(ListItemModel(title: Localized.Transfer.payWith, subtitle: symbol), selectable: selectable)
        case .details:
            .empty
        }
    }
}

// MARK: - Private

extension ConfirmRowViewModel {
    private func recipientItem(destination: GemConfirmDestination, addressName: AddressName?, memo: String?, chain: Chain, link: BlockExplorerLink) -> AddressListItemViewModel {
        let (name, address): (String?, String) = switch destination {
        case let .recipient(name, address): (name, address)
        case let .contract(address): (addressName?.name, address)
        case let .validator(name, address): (name, address)
        case let .resource(resource): (resource.toPrimitives().title, "")
        case let .provider(name, address): (name, address)
        }
        return AddressListItemViewModel(
            title: destination.title,
            account: SimpleAccount(
                name: name,
                chain: chain,
                address: address,
                memo: memo,
                assetImage: contactImage(addressName),
                addressType: addressName?.type,
            ),
            mode: .nameOrAddress,
            addressLink: link,
            onSelect: selectAction(destination: destination, chainAddress: ChainAddress(chain: chain, address: address)),
        )
    }

    private func selectAction(destination: GemConfirmDestination, chainAddress: ChainAddress) -> (@MainActor @Sendable () -> Void)? {
        guard let onSelectAddress else { return nil }
        switch destination {
        case .recipient, .contract, .validator, .provider:
            return { onSelectAddress(chainAddress) }
        case .resource:
            return nil
        }
    }

    private func contactImage(_ addressName: AddressName?) -> AssetImage? {
        guard let addressName else { return nil }
        switch addressName.type {
        case .contact:
            return AssetImage(
                type: .text(String(addressName.name.prefix(2))),
                imageURL: addressName.imageUrl.map { ImageSource($0).url },
            )
        case .address, .contract, .validator, .internalWallet:
            return nil
        }
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
