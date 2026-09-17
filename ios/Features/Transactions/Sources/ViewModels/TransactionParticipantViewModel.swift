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
    private let onSelectAddress: (@MainActor @Sendable (ChainAddress) -> Void)?

    init(
        participant: GemTransactionParticipant?,
        chain: Chain,
        memo: String?,
        onAddContact: ((AddContactType) -> Void)? = nil,
        onSelectAddress: (@MainActor @Sendable (ChainAddress) -> Void)? = nil,
    ) {
        self.participant = participant
        self.chain = chain
        self.memo = memo
        self.onAddContact = onAddContact
        self.onSelectAddress = onSelectAddress
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
        let name = participant.name?.toPrimitives()
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
                title: participant.role.title,
                account: account,
                addressLink: participant.link.toPrimitives(),
                onAddContact: participant.canAddContact ? onAddContact : nil,
                onSelect: selectAction(chainAddress: ChainAddress(chain: chain, address: participant.address)),
            ),
        )
    }

    private func selectAction(chainAddress: ChainAddress) -> (@MainActor @Sendable () -> Void)? {
        guard let onSelectAddress else { return nil }
        return { onSelectAddress(chainAddress) }
    }

}
