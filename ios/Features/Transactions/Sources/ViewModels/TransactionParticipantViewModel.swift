// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemTransactionParticipant
import enum Gemstone.GemTransactionParticipantRole
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents

struct TransactionParticipantViewModel {
    private let transactionViewModel: TransactionViewModel
    private let participant: GemTransactionParticipant?
    private let onAddContact: ((AddContactType) -> Void)?

    init(
        transactionViewModel: TransactionViewModel,
        participant: GemTransactionParticipant?,
        onAddContact: ((AddContactType) -> Void)? = nil,
    ) {
        self.transactionViewModel = transactionViewModel
        self.participant = participant
        self.onAddContact = onAddContact
    }
}

// MARK: - ItemModelProvidable

extension TransactionParticipantViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        switch transactionViewModel.transaction.transaction.type {
        case .stakeFreeze, .stakeUnfreeze: resourceItemModel
        case .transfer, .transferNFT, .tokenApproval, .smartContractCall, .stakeDelegate, .earnDeposit, .earnWithdraw,
             .swap, .stakeUndelegate, .stakeRedelegate, .stakeRewards, .stakeWithdraw, .assetActivation,
             .perpetualOpenPosition, .perpetualClosePosition, .perpetualModifyPosition: participantItemModel
        }
    }
}

// MARK: - Private

extension TransactionParticipantViewModel {
    private var participantItemModel: TransactionItemModel {
        guard let participant else { return .empty }

        let addressName = transactionViewModel.getAddressName(address: participant.address)
        let account = SimpleAccount(
            name: addressName?.name,
            chain: transactionViewModel.transaction.transaction.assetId.chain,
            address: participant.address,
            memo: transactionViewModel.transaction.transaction.memo,
            assetImage: nil,
            addressType: addressName?.type,
        )

        return .participant(
            TransactionParticipantItemModel(
                title: title(for: participant.role),
                account: account,
                addressLink: BlockExplorerLink(participant.link),
                onAddContact: canAddContact(addressName: addressName) ? onAddContact : nil,
            ),
        )
    }

    private func canAddContact(addressName: AddressName?) -> Bool {
        guard addressName == nil else { return false }
        let type = transactionViewModel.transaction.transaction.type
        return type == .transfer || type == .transferNFT
    }

    private var resourceItemModel: TransactionItemModel {
        guard let resourceType = transactionViewModel.transaction.transaction.metadata?.decode(TransactionResourceTypeMetadata.self)?.resourceType else {
            return .empty
        }
        let resourceTitle = ResourceViewModel(resource: resourceType).title
        return .listItem(ListItemModel(title: Localized.Stake.resource, subtitle: resourceTitle))
    }

    private func title(for role: GemTransactionParticipantRole) -> String {
        switch role {
        case .sender: Localized.Transaction.sender
        case .recipient: Localized.Transaction.recipient
        case .contract: Localized.Asset.contract
        case .validator: Localized.Stake.validator
        case .provider: Localized.Common.provider
        }
    }
}
