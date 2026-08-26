// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.EvmTransactionKind
import GemstonePrimitives
import Primitives
import Testing
@testable import WalletConnectorService

struct WalletConnectorTransactionTests {
    @Test
    func evmTransactionKindData() throws {
        let approval = ApprovalData(token: "token", spender: "spender", value: "100", isUnlimited: false)
        let approvalKind = try EvmTransactionKind.tokenApproval(approval: approval.json()).map()
        let transferKind = try EvmTransactionKind.transfer.map()
        let contractCallKind = try EvmTransactionKind.contractCall.map()

        #expect(transferKind.transactionType == .transfer)
        #expect(transferKind.approvalData == nil)
        #expect(contractCallKind.transactionType == .smartContractCall)
        #expect(contractCallKind.approvalData == nil)
        #expect(approvalKind.transactionType == .tokenApproval)
        #expect(approvalKind.approvalData == approval)
    }
}
