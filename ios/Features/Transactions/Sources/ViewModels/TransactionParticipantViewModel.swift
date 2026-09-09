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
    private let participant: GemTransactionParticipant?
    private let chain: Chain
    private let memo: String?
    private let onAddContact: ((AddContactType) -> Void)?

    init(
        participant: GemTransactionParticipant?,
        chain: Chain,
        memo: String?,
        onAddContact: ((AddContactType) -> Void)? = nil,
    ) {
        self.participant = participant
        self.chain = chain
        self.memo = memo
        self.onAddContact = onAddContact
    }
}

extension TransactionParticipantViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        guard let participant else { return .empty }
        return participantItemModel(participant)
    }
}

extension TransactionParticipantViewModel {
    private func participantItemModel(_ participant: GemTransactionParticipant) -> TransactionItemModel {
        let name = participant.name?.map()
        let account = SimpleAccount(
            name: name?.name,
            chain: chain,
            address: participant.address,
            memo: memo,
            assetImage: nil,
            addressType: name?.type,
        )
        return .participant(
            TransactionParticipantItemModel(
                title: title(for: participant.role),
                account: account,
                addressLink: participant.link.map(),
                onAddContact: participant.canAddContact ? onAddContact : nil,
            ),
        )
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
