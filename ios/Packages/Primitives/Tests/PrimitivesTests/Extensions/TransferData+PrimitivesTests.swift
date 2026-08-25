// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Testing

struct TransferDataTypeTests {
    @Test
    func genericEncodedTransaction() throws {
        let asset = Asset.mockSolana()
        let metadata = ApplicationMetadata.mock(source: .payment)
        let data = TransferData(
            asset: asset,
            metadata: metadata,
            transaction: "encoded-transaction",
            memo: "payment memo",
            outputType: .encodedTransaction,
            outputAction: .send,
            transactionType: .transfer,
        )

        guard case let .generic(mappedAsset, mappedMetadata, extra) = data.type else {
            Issue.record("Expected generic transfer data")
            return
        }
        #expect(mappedAsset == asset)
        #expect(mappedMetadata == metadata)
        #expect(extra.data.flatMap { String(data: $0, encoding: .utf8) } == "encoded-transaction")
        #expect(extra.outputType == .encodedTransaction)
        #expect(extra.outputAction == .send)
        #expect(extra.transactionType == .transfer)
        #expect(try data.encodedTransaction() == "encoded-transaction")
        #expect(data.recipientData.recipient.address.isEmpty)
        #expect(data.recipientData.recipient.memo == "payment memo")
        #expect(data.value == .zero)
    }

    @Test
    func approvalDataMatchesTransactionType() throws {
        let approval = ApprovalData.mock()
        let swap = TransferDataType.swap(
            .mock(),
            .mock(),
            .mock(data: .mock(approval: approval)),
        )

        #expect(try swap.approvalData(for: .tokenApproval) == approval)
        #expect(try swap.approvalData(for: .swap) == nil)
        #expect(throws: Error.self) {
            try TransferDataType.swap(.mock(), .mock(), .mock()).approvalData(for: .tokenApproval)
        }

        let generic = TransferDataType.generic(
            asset: .mock(),
            metadata: .mock(),
            extra: .mock(transactionType: .tokenApproval, approval: approval),
        )
        #expect(try generic.approvalData(for: .tokenApproval) == approval)
    }

    @Test
    func shouldIgnoreValueCheck() {
        #expect(TransferData.mock(type: .transferNft(.mock())).type.shouldIgnoreValueCheck == true)
        #expect(TransferData.mock(type: .stake(.mock(), .stake(.mock()))).type.shouldIgnoreValueCheck == true)
        #expect(TransferData.mock(type: .account(.mock(), .activate)).type.shouldIgnoreValueCheck == true)
        #expect(TransferData.mock(type: .transfer(.mock())).type.shouldIgnoreValueCheck == false)

        #expect(TransferData.mock(type: .deposit(.mock())).type.shouldIgnoreValueCheck == false)
        #expect(TransferData.mock(type: .perpetual(.mock(), .open(.mock(direction: .long, assetIndex: 0, price: "100", size: "1")))).type.shouldIgnoreValueCheck == true)
        #expect(
            TransferData
                .mock(
                    type: .perpetual(.mock(), .close(.mock(direction: .long, assetIndex: 0, price: "100", size: "1"))),
                ).type.shouldIgnoreValueCheck == true,
        )
    }

    @Test
    func perpetualOpenTransactionType() {
        let asset = Asset.mock()

        let openType = TransferDataType.perpetual(asset, .mockOpen())
        let increaseType = TransferDataType.perpetual(asset, .mockIncrease())

        #expect(openType.transactionType == .perpetualOpenPosition)
        #expect(increaseType.transactionType == .perpetualOpenPosition)
    }

    @Test
    func perpetualCloseTransactionType() {
        let asset = Asset.mock()

        let closeType = TransferDataType.perpetual(asset, .mockClose())
        let reduceType = TransferDataType.perpetual(asset, .mockReduce())

        #expect(closeType.transactionType == .perpetualClosePosition)
        #expect(reduceType.transactionType == .perpetualClosePosition)
    }

    @Test
    func perpetualModifyTransactionType() {
        let asset = Asset.mock()
        let modifyType = TransferDataType.perpetual(asset, .mockModify())

        #expect(modifyType.transactionType == .perpetualModifyPosition)
    }

    @Test
    func withGasLimit() throws {
        let type = TransferDataType.swap(.mock(), .mock(), .mock(data: SwapQuoteData(to: "", dataType: .contract, value: "", data: "", memo: nil, approval: nil, gasLimit: "0")))

        let (_, _, swapBefore) = try type.swap()
        #expect(swapBefore.data.gasLimit == "0")

        let (_, _, swapAfter) = try type.withGasLimit("21000").swap()
        #expect(swapAfter.data.gasLimit == "21000")
    }

    @Test
    func freezeMetadata() {
        let bandwidth = TransferDataType.stake(.mock(), .freeze(.bandwidth))
        let energy = TransferDataType.stake(.mock(), .unfreeze(.energy))

        #expect(bandwidth.metadata == .object(["resourceType": .string("bandwidth")]))
        #expect(energy.metadata == .object(["resourceType": .string("energy")]))
    }
}
