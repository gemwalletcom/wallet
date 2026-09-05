// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemConfirmDestination
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents

struct ConfirmRecipientViewModel {
    private let destination: GemConfirmDestination?
    private let chain: Chain
    private let memo: String?
    private let addressName: AddressName?
    private let addressLink: BlockExplorerLink

    init(
        destination: GemConfirmDestination?,
        chain: Chain,
        memo: String?,
        addressName: AddressName?,
        addressLink: BlockExplorerLink,
    ) {
        self.destination = destination
        self.chain = chain
        self.memo = memo
        self.addressName = addressName
        self.addressLink = addressLink
    }
}

// MARK: - ItemModelProvidable

extension ConfirmRecipientViewModel: ItemModelProvidable {
    var itemModel: ConfirmTransferItemModel {
        guard let destination else { return .empty }
        let (title, name, address): (String, String?, String) = switch destination {
        case let .recipient(name, address): (Localized.Transfer.Recipient.title, addressName?.name ?? name, address)
        case let .contract(address): (Localized.Asset.contract, addressName?.name, address)
        case let .validator(name, address): (Localized.Stake.validator, name, address)
        case let .resource(resource): (Localized.Stake.resource, ResourceViewModel(resource: resource.map()).title, "")
        case let .provider(name, address): (Localized.Common.provider, name, address)
        }
        return .recipient(
            AddressListItemViewModel(
                title: title,
                account: SimpleAccount(
                    name: name,
                    chain: chain,
                    address: address,
                    memo: memo,
                    assetImage: addressNameImage,
                    addressType: addressName?.type,
                ),
                mode: .nameOrAddress,
                addressLink: addressLink,
            ),
        )
    }
}

// MARK: - Private

extension ConfirmRecipientViewModel {
    private var addressNameImage: AssetImage? {
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
