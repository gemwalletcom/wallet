// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import Primitives
import PrimitivesComponents

struct ConfirmRecipientViewModel {
    private let model: TransferDataViewModel
    private let addressName: AddressName?
    private let addressLink: BlockExplorerLink
    private let onAddContact: ((AddContactType) -> Void)?

    init(
        model: TransferDataViewModel,
        addressName: AddressName?,
        addressLink: BlockExplorerLink,
        onAddContact: ((AddContactType) -> Void)? = nil,
    ) {
        self.model = model
        self.addressName = addressName
        self.addressLink = addressLink
        self.onAddContact = onAddContact
    }
}

// MARK: - ItemModelProvidable

extension ConfirmRecipientViewModel: ItemModelProvidable {
    var itemModel: ConfirmTransferItemModel {
        guard showRecipient else { return .empty }
        return .recipient(
            AddressListItemViewModel(
                title: recipientTitle,
                account: SimpleAccount(
                    name: addressName?.name ?? model.recipient.name,
                    chain: model.chain,
                    address: model.recipient.address,
                    memo: model.recipient.memo,
                    assetImage: addressNameImage,
                    addressType: addressName?.type,
                ),
                mode: .nameOrAddress,
                addressLink: addressLink,
                onAddContact: addressName?.type == nil ? onAddContact : nil,
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

    private var recipientTitle: String {
        switch model.type {
        case .swap: Localized.Common.provider
        case let .stake(_, stakeType):
            switch stakeType {
            case .stake, .unstake, .redelegate, .rewards, .withdraw: Localized.Stake.validator
            case .freeze, .unfreeze: Localized.Stake.resource
            }
        case .generic:
            switch model.type.outputAction {
            case .sign: Localized.Asset.contract
            case .send: Localized.Transfer.Recipient.title
            }
        case .earn: Localized.Common.provider
        case .transfer, .deposit, .withdrawal, .transferNft, .tokenApprove, .account, .perpetual: Localized.Transfer.Recipient.title
        }
    }

    private var showRecipient: Bool {
        guard !model.recipient.address.isEmpty else { return false }

        return switch model.type {
        case let .stake(_, stakeType):
            switch stakeType {
            case .stake, .unstake, .redelegate, .withdraw, .rewards: true
            case .freeze, .unfreeze: true
            }
        case .account,
             .swap,
             .perpetual: false
        case .earn: true
        case .generic:
            switch model.type.outputAction {
            case .sign: false
            case .send: true
            }
        case .transfer,
             .transferNft,
             .deposit,
             .withdrawal,
             .tokenApprove: true
        }
    }
}
