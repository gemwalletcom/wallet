// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Testing
@testable import Transfer
import WalletSessionService
import WalletSessionServiceTestKit

@MainActor
struct RecipientSceneViewModelTests {
    @Test
    func tittle() {
        #expect(RecipientSceneViewModel.mock().tittle == "Recipient")
    }

    @Test
    func recipientField() {
        #expect(RecipientSceneViewModel.mock().recipientField == "Address or Name")
    }

    @Test
    func memoField() {
        #expect(RecipientSceneViewModel.mock().memoField == "Memo")
    }

    @Test
    func actionButtonTitle() {
        #expect(RecipientSceneViewModel.mock().actionButtonTitle == "Continue")
    }

    @Test
    func showMemo() {
        #expect(RecipientSceneViewModel.mock(asset: .mock(id: AssetId(chain: .cosmos, tokenId: nil))).showMemo == true)
        #expect(RecipientSceneViewModel.mock(asset: .mock(id: AssetId(chain: .ton, tokenId: nil))).showMemo == true)
        #expect(RecipientSceneViewModel.mock(asset: .mock(id: AssetId(chain: .bitcoin, tokenId: nil))).showMemo == false)
        #expect(RecipientSceneViewModel.mock(asset: .mockEthereum()).showMemo == false)
    }

    @Test
    func shouldShowInputActions() {
        let model = RecipientSceneViewModel.mock()
        #expect(model.addressInputModel.shouldShowInputActions == true)

        model.addressInputModel.text = "0x123"
        #expect(model.addressInputModel.shouldShowInputActions == false)
    }

    @Test
    func actionButtonState() {
        let model = RecipientSceneViewModel.mock()

        #expect(model.actionButtonState == .disabled)

        model.addressInputModel.text = "0x1234567890123456789012345678901234567890"
        _ = model.addressInputModel.update()

        #expect(model.actionButtonState == .normal)

        model.addressInputModel.text = "invalid"
        _ = model.addressInputModel.update()

        #expect(model.actionButtonState == .disabled)

        model.addressInputModel.nameRecordViewModel.state = .loading
        #expect(model.actionButtonState == .disabled)

        model.addressInputModel.text = "test.eth"
        model.addressInputModel.nameRecordViewModel.state = .complete(NameRecord.mock())
        #expect(model.actionButtonState == .normal)
    }

    @Test
    func onContinueUsesChecksumAddress() {
        let address = "0x5615e8ab93b9d695b6d4d6545f7792aa59e1069a"
        let checksummed = "0x5615E8AB93b9d695b6d4d6545f7792aA59e1069a"
        var recipientData: RecipientData?
        let model = RecipientSceneViewModel.mock(onRecipientDataAction: { recipientData = $0 })

        model.addressInputModel.text = " \n\(address)\r "
        model.onContinue()

        #expect(recipientData?.recipient.address == checksummed)

        recipientData = nil
        model.addressInputModel.text = "test.eth"
        model.addressInputModel.nameRecordViewModel.state = .complete(.mock(address: address))
        model.onContinue()

        #expect(recipientData?.recipient.address == checksummed)
    }

    @Test
    func destination_confirm() throws {
        let asset = Asset.mockEthereum()
        let model = RecipientSceneViewModel.mock(asset: asset, type: .mockAsset(asset))
        let address = "0x5615e8ab93b9d695b6d4d6545f7792aa59e1069a"
        let checksummed = "0x5615E8AB93b9d695b6d4d6545f7792aA59e1069a"

        let payment = PaymentRequest(
            address: " \n\(address)\r ",
            amount: "1.234",
            memo: nil,
            assetId: nil,
        )

        let result = try PaymentTransfer(asset: model.asset).destination(for: payment)

        switch result {
        case let .confirm(data):
            #expect(data.recipientData.recipient.address == checksummed)
            #expect(data.amount == .exact(BigInt("1234000000000000000")))
        case .recipient:
            Issue.record("Expected confirm but got recipient")
        }
    }

    @Test
    func destination_recipient() throws {
        let model = RecipientSceneViewModel.mock()

        let payment = PaymentRequest(
            address: "0x123",
            amount: nil,
            memo: "test memo",
            assetId: nil,
        )

        let result = try PaymentTransfer(asset: model.asset).destination(for: payment)

        switch result {
        case let .recipient(data):
            #expect(data.recipient.address == payment.address)
            #expect(data.recipient.memo == payment.memo)
            #expect(data.amount == nil)
        case .confirm:
            Issue.record("Expected recipient but got confirm")
        }
    }

    @Test
    func onHandleScan_keepsAmount() {
        let asset = Asset.mockEthereum()
        let model = RecipientSceneViewModel.mock(asset: asset, type: .mockAsset(asset))

        model.onHandleScan("ethereum:0x123?amount=1.5", for: .address)
        model.onChangeAddressText("", new: model.addressInputModel.text)

        #expect(model.scanned?.amount == "1.5")

        model.onChangeAddressText("", new: "0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326")

        #expect(model.scanned == nil)
    }

    @Test
    func prefilledRecipient_keepsAmount() {
        let asset = Asset.mockEthereum()
        let address = "0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326"
        let model = RecipientSceneViewModel.mock(
            asset: asset,
            type: .mockAsset(asset),
            recipient: RecipientData(
                recipient: Recipient(name: .none, address: address, memo: "12345"),
                amount: "10",
            ),
        )

        #expect(model.addressInputModel.text == address)
        #expect(model.memo == "12345")
        #expect(model.scanned?.amount == "10")

        model.onChangeAddressText(address, new: "0x5615e8ab93b9d695b6d4d6545f7792aa59e1069a")

        #expect(model.scanned == nil)
    }

    @Test
    func destination_belowSmallestUnit() throws {
        let asset = Asset.mockEthereum()
        let model = RecipientSceneViewModel.mock(asset: asset, type: .mockAsset(asset))

        let payment = PaymentRequest(
            address: "0x5615e8ab93b9d695b6d4d6545f7792aa59e1069a",
            amount: "0.0000000000000000001",
            memo: nil,
            assetId: nil,
        )

        let result = try PaymentTransfer(asset: model.asset).destination(for: payment)

        switch result {
        case let .recipient(data):
            #expect(data.amount == payment.amount)
        case .confirm:
            Issue.record("A nineteenth decimal is not signable as ETH")
        }
    }

    @Test
    func nftAssetImage() {
        let nftAsset = NFTAsset.mock(id: NFTAssetId(chain: .ethereum, contractAddress: "0x123", tokenId: "1"))
        let image = RecipientSceneViewModel.mock().nftAssetImage(for: nftAsset)
        #expect(image.imageURL?.absoluteString.contains("ethereum_0x123::1") == true)
    }
}

// MARK: - Mocks

extension RecipientSceneViewModel {
    static func mock(
        wallet: Wallet = .mock(),
        asset: Asset = .mockEthereum(),
        type: RecipientAssetType = .mockAsset(),
        recipient: RecipientData? = .none,
        onRecipientDataAction: RecipientDataAction = nil,
        onTransferAction: TransferDataAction = nil,
    ) -> RecipientSceneViewModel {
        RecipientSceneViewModel(
            wallet: wallet,
            asset: asset,
            walletSessionService: WalletSessionService.mock(),
            nameService: .mock(),
            type: type,
            recipient: recipient,
            onRecipientDataAction: onRecipientDataAction,
            onTransferAction: onTransferAction,
        )
    }
}
