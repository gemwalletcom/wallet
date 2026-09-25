// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemPaymentRecipient
import enum Gemstone.GemRecipientErrorDisplay
import struct Gemstone.GemTransferData
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

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

        #expect(model.actionButtonState == .normal)

        model.addressInputModel.text = "invalid"

        #expect(model.actionButtonState == .disabled)

        model.addressInputModel.nameRecordViewModel.state = .loading(name: "test.eth", chain: Chain.ethereum.toGem())
        #expect(model.actionButtonState == .disabled)

        model.addressInputModel.text = "test.eth"
        model.addressInputModel.nameRecordViewModel.state = .complete(record: NameRecord.mock().toGem())
        #expect(model.actionButtonState == .normal)
    }

    @Test
    func onContinueUsesChecksumAddress() {
        let address = "0x5615e8ab93b9d695b6d4d6545f7792aa59e1069a"
        let checksummed = "0x5615E8AB93b9d695b6d4d6545f7792aA59e1069a"
        var recipientAddress: String?
        let model = RecipientSceneViewModel.mock(onNavigate: { step in
            guard case let .amount(input) = step, case let .transfer(recipient) = input.type else { return }
            recipientAddress = recipient.recipient.address
        })

        model.addressInputModel.text = " \n\(address)\r "
        model.onContinue()

        #expect(recipientAddress == checksummed)

        recipientAddress = nil
        model.addressInputModel.text = "test.eth"
        model.addressInputModel.nameRecordViewModel.state = .complete(record: NameRecord.mock(address: address).toGem())
        model.onContinue()

        #expect(recipientAddress == checksummed)
    }

    @Test
    func mismatchedTokenScanShowsNetworkError() {
        var didNavigate = false
        let model = RecipientSceneViewModel.mock(asset: .mockSolanaUSDC(), onNavigate: { _ in didNavigate = true })

        model.onHandleScan("ethereum:0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326", for: .address)

        #expect(model.addressInputModel.inputModel.error?.localizedDescription == GemRecipientErrorDisplay.invalidAddress(network: "Solana").errorDescription)
        #expect(didNavigate == false)
    }

    @Test
    func invalidRecipientShowsNetworkError() {
        var didNavigate = false
        let model = RecipientSceneViewModel.mock(onNavigate: { _ in didNavigate = true })

        model.onSelectRecipient(.mock(address: "invalid address"))

        #expect(model.addressInputModel.text == "invalid address")
        #expect(model.addressInputModel.inputModel.error?.localizedDescription == GemRecipientErrorDisplay.invalidAddress(network: "Ethereum").errorDescription)
        #expect(didNavigate == false)
    }

    @Test
    func onHandleScanKeepsAmount() {
        let asset = Asset.mockEthereum()
        let model = RecipientSceneViewModel.mock(asset: asset, type: .asset(asset: asset.toGem()))

        model.onHandleScan("ethereum:0x123?amount=1.5", for: .address)
        model.onChangeAddressText("", new: model.addressInputModel.text)

        #expect(model.session.payment?.amount == "1.5")

        model.onChangeAddressText("", new: "0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326")

        #expect(model.session.payment == nil)
    }

    @Test
    func recipientDataKeepsAmount() {
        let asset = Asset.mockEthereum()
        let address = "0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326"
        let model = RecipientSceneViewModel.mock(
            asset: asset,
            type: .asset(asset: asset.toGem()),
            recipient: .mock(recipient: .mock(address: address, memo: "12345"), amount: "10"),
        )

        #expect(model.addressInputModel.text == address)
        #expect(model.memo == "12345")
        #expect(model.session.payment?.amount == "10")

        model.onChangeAddressText(address, new: "0x5615e8ab93b9d695b6d4d6545f7792aa59e1069a")

        #expect(model.session.payment == nil)
    }

    @Test
    func onHandleScanWithAmountGoesStraightToConfirm() {
        let asset = Asset.mockEthereum()
        var transfer: GemTransferData?
        let model = RecipientSceneViewModel.mock(asset: asset, type: .asset(asset: asset.toGem()), onNavigate: {
            if case let .confirm(data) = $0 {
                transfer = data
            }
        })

        model.onHandleScan("ethereum:0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326?amount=1.5", for: .address)

        #expect(transfer?.recipient.address == "0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326")
        #expect(model.session.payment == nil)
    }

    @Test
    func onHandleScanForAnNftOnlyFillsTheRecipient() {
        var transfer: GemTransferData?
        let model = RecipientSceneViewModel.mock(type: .nft(nftAsset: NFTAsset.mock(chain: .ethereum).toGem()), onNavigate: {
            if case let .confirm(data) = $0 {
                transfer = data
            }
        })

        model.onHandleScan("ethereum:0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326?amount=1.5", for: .address)

        #expect(transfer == nil)
        #expect(model.addressInputModel.text == "0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326")
    }

    @Test
    func onContinueForAnNftConfirmsATransferOfTheAsset() {
        let nftAsset = NFTAsset.mock(chain: .ethereum)
        var transfer: GemTransferData?
        let model = RecipientSceneViewModel.mock(type: .nft(nftAsset: nftAsset.toGem()), onNavigate: {
            if case let .confirm(data) = $0 {
                transfer = data
            }
        })

        model.addressInputModel.text = "0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326"
        model.onContinue()

        guard case let .transferNft(_, found)? = transfer?.inputType else {
            Issue.record("Expected an nft transfer")
            return
        }
        #expect(found.id == nftAsset.toGem().id)
        #expect(transfer?.value == .zero)
        #expect(transfer?.recipient.address == "0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326")
    }

    @Test
    func nftAssetImage() {
        let nftAsset = NFTAsset.mock(id: NFTAssetId(chain: .ethereum, contractAddress: "0x123", tokenId: "1"))
        let image = RecipientSceneViewModel.mock().nftAssetImage(for: nftAsset)
        #expect(image.imageURL?.absoluteString.contains("ethereum_0x123::1") == true)
    }
}
