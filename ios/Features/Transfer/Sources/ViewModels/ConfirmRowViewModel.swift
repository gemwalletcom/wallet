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
    private let content: GemConfirmRowContent?

    init(content: GemConfirmRowContent?) {
        self.content = content
    }
}

// MARK: - ItemModelProvidable

extension ConfirmRowViewModel: ItemModelProvidable {
    var itemModel: ConfirmTransferItemModel {
        switch content {
        case let .app(name, iconUrl):
            .app(
                ListItemModel(
                    title: Localized.WalletConnect.app,
                    subtitle: name,
                    imageStyle: .list(assetImage: iconUrl.map { AssetImage(imageURL: URL(string: $0)) }),
                ),
            )
        case let .sender(wallet):
            .sender(
                ListItemModel(
                    title: Localized.Common.wallet,
                    subtitle: wallet.name,
                    imageStyle: .list(assetImage: wallet.avatarImage),
                ),
            )
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
        case let .network(chain, name):
            .network(
                ListItemModel(
                    title: Localized.Transfer.network,
                    subtitle: name,
                    imageStyle: .list(assetImage: AssetIdViewModel(assetId: AssetId(chain: Chain(core: chain), tokenId: nil)).networkAssetImage),
                ),
            )
        case let .memo(memo):
            .memo(MemoViewModel(memo: memo).listItemModel)
        case .details, .none:
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
        )
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
    var item: ConfirmTransferItem {
        switch self {
        case .app: .app
        case .sender: .sender
        case .recipient: .recipient
        case .network: .network
        case .memo: .memo
        case .details: .details
        }
    }
}
