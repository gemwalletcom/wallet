// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.EvmTransactionKind
import struct Gemstone.GemApprovalData
import Primitives
import Testing
@testable import WalletConnectorService

struct WalletConnectorTransactionTests {
    @Test
    func evmTransactionKindData() {
        let approval = ApprovalData(token: "token", spender: "spender", value: "100", isUnlimited: false)
        let approvalKind = EvmTransactionKind.tokenApproval(approval: GemApprovalData(
            token: approval.token,
            spender: approval.spender,
            value: approval.value,
            isUnlimited: approval.isUnlimited,
        )).map()
        let transferKind = EvmTransactionKind.transfer.map()
        let contractCallKind = EvmTransactionKind.contractCall.map()

        #expect(transferKind.transactionType == .transfer)
        #expect(transferKind.approvalData == nil)
        #expect(contractCallKind.transactionType == .smartContractCall)
        #expect(contractCallKind.approvalData == nil)
        #expect(approvalKind.transactionType == .tokenApproval)
        #expect(approvalKind.approvalData == approval)
    }
}
