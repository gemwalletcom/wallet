// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Foundation
import enum Gemstone.FeePriority
import class Gemstone.GemAssetConfigService
import struct Gemstone.GemBalanceRequirement
import enum Gemstone.GemConfirmError
import struct Gemstone.GemConfirmSimulation
import struct Gemstone.GemFeeRate
import protocol Gemstone.GemNameServiceProtocol
import struct Gemstone.GemTransferData
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Store
import Testing
@testable import Transfer
import TransferTestKit

@MainActor
struct ConfirmTransferSceneViewModelTests {
    @Test
    func paymentHeaderAppearsAfterLoading() {
        let data = GemTransferData.mockPayment()
        let model = ConfirmTransferSceneViewModel.mock(data: data)

        #expect(model.isHeaderVisible == false)
        model.state.transaction = .data(.mock())
        #expect(model.isHeaderVisible == true)
    }

    @Test
    func itemModelReturnsNonEmpty() {
        let model = ConfirmTransferSceneViewModel.mock()

        verifyNonEmpty(model.itemModel(for: .header))
        verifyNonEmpty(model.itemModel(for: .sender))
        verifyNonEmpty(model.itemModel(for: .network))
        verifyNonEmpty(model.itemModel(for: .recipient))
        verifyNonEmpty(model.itemModel(for: .networkFee))
    }

    @Test
    func headerItemModel() {
        let model = ConfirmTransferSceneViewModel.mock(
            data: .mock(type: .transfer(.mockEthereum())),
        )
        let headerItem = model.itemModel(for: .header) as? ConfirmHeaderViewModel

        if case .header = headerItem?.itemModel {
            // Expected header item
        } else {
            Issue.record("Expected header item model")
        }
    }

    @Test
    func appItemModel() {
        let model = ConfirmTransferSceneViewModel.mock()
        let appItem = model.itemModel(for: .app) as? ConfirmAppViewModel

        if case .empty = appItem?.itemModel {
            // Expected empty for non-generic transfer
        } else {
            Issue.record("Expected empty app item model")
        }

        let modelWithWebsite = ConfirmTransferSceneViewModel.mock(
            data: .mock(type: .generic(asset: .mock(), metadata: .mock(name: "Gem Wallet", url: "https://example.com"), extra: .mock())),
        )
        let appItemWithWebsite = modelWithWebsite.itemModel(for: .app) as? ConfirmAppViewModel

        if case let .app(listItem) = appItemWithWebsite?.itemModel {
            #expect(listItem.subtitle == "Gem Wallet")
        } else {
            Issue.record("Expected app item model")
        }
    }

    @Test
    func title() {
        #expect(ConfirmTransferSceneViewModel.mock(data: .mock(type: .transfer(.mock()))).title == Localized.Transfer.Send.title)
        // #expect(ConfirmTransferViewModel.mock(data: .mock(type: .transferNft(.mock()))).title == Localized.Transfer.Send.title)
        #expect(ConfirmTransferSceneViewModel.mock(data: .mock(type: .swap(.mock(), .mock(), .mock()))).title == Localized.Wallet.swap)
        #expect(ConfirmTransferSceneViewModel.mock(data: .mock(type: .tokenApprove(.mock(), .mock()))).title == Localized.Wallet.swap)
        #expect(ConfirmTransferSceneViewModel.mock(data: .mock(type: .generic(asset: .mock(), metadata: .mock(), extra: .mock()))).title == Localized.Transfer.reviewRequest)
    }

    @Test
    func senderItemModel() {
        let model = ConfirmTransferSceneViewModel.mock()
        let senderItem = model.itemModel(for: .sender) as? ConfirmSenderViewModel

        if case let .sender(listItem) = senderItem?.itemModel {
            #expect(listItem.title == Localized.Wallet.title)
        } else {
            Issue.record("Expected sender item model")
        }
    }

    @Test
    func recipientItemModel() {
        let address = "0x1234567890123456789012345678901234567890"
        let model = ConfirmTransferSceneViewModel.mock(data: .mock(
            type: .transfer(.mock()),
            recipient: .mock(address: address),
        ))
        let recipientItem = model.itemModel(for: .recipient) as? ConfirmRecipientViewModel

        if case let .recipient(addressViewModel) = recipientItem?.itemModel {
            #expect(addressViewModel.account.address == address)
            #expect(addressViewModel.account.name == nil)
        } else {
            Issue.record("Expected recipient item model")
        }
    }

    @Test
    func recipientNameItemModel() async {
        let address = "bc1qml9s2f9k8wc0882x63lyplzp97srzg2c39fyaw"
        let model = ConfirmTransferSceneViewModel.mock(
            data: .mock(
                type: .transfer(.mock()),
                recipient: .mock(address: address),
            ),
            load: .success(.mock(addressName: .mock(chain: .bitcoin, address: address, name: "Bitcoin"))),
        )
        await model.load()
        let recipientItem = model.itemModel(for: .recipient) as? ConfirmRecipientViewModel

        if case let .recipient(addressViewModel) = recipientItem?.itemModel {
            #expect(addressViewModel.account.address == address)
            #expect(addressViewModel.account.name == "Bitcoin")
        } else {
            Issue.record("Expected recipient item model")
        }
    }

    @Test
    func recipientNameItemModelUsesStoredAddress() async {
        let checksummedAddress = "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4"
        let model = ConfirmTransferSceneViewModel.mock(
            data: .mock(
                type: .transfer(.mockEthereum()),
                recipient: .mock(address: checksummedAddress),
            ),
            load: .success(.mock(addressName: .mock(chain: .ethereum, address: checksummedAddress, name: "Uniswap"))),
        )
        await model.load()
        let recipientItem = model.itemModel(for: .recipient) as? ConfirmRecipientViewModel

        if case let .recipient(addressViewModel) = recipientItem?.itemModel {
            #expect(addressViewModel.account.address == checksummedAddress)
            #expect(addressViewModel.account.name == "Uniswap")
        } else {
            Issue.record("Expected recipient item model")
        }
    }

    @Test
    func networkItemModel() {
        let ethModel = ConfirmTransferSceneViewModel.mock(data: .mock(type: .transfer(.mockEthereum())))
        let ethNetworkItem = ethModel.itemModel(for: .network) as? ConfirmNetworkViewModel

        if case let .network(listItem) = ethNetworkItem?.itemModel {
            #expect(listItem.subtitle == "Ethereum")
        } else {
            Issue.record("Expected network item model for ETH")
        }

        let usdtModel = ConfirmTransferSceneViewModel.mock(data: .mock(type: .transfer(.mockEthereumUSDT())))
        let usdtNetworkItem = usdtModel.itemModel(for: .network) as? ConfirmNetworkViewModel

        if case let .network(listItem) = usdtNetworkItem?.itemModel {
            #expect(listItem.subtitle == "Ethereum (ERC20)")
        } else {
            Issue.record("Expected network item model for USDT")
        }

        let genericEthModel = ConfirmTransferSceneViewModel.mock(data: .mock(type: .generic(asset: .mockEthereum(), metadata: .mock(), extra: .mock())))
        let genericEthNetworkItem = genericEthModel.itemModel(for: .network) as? ConfirmNetworkViewModel

        if case let .network(listItem) = genericEthNetworkItem?.itemModel {
            #expect(listItem.subtitle == "Ethereum")
        } else {
            Issue.record("Expected network item model for generic ETH")
        }

        let genericUsdtModel = ConfirmTransferSceneViewModel.mock(data: .mock(type: .generic(asset: .mockEthereumUSDT(), metadata: .mock(), extra: .mock())))
        let genericUsdtNetworkItem = genericUsdtModel.itemModel(for: .network) as? ConfirmNetworkViewModel

        if case let .network(listItem) = genericUsdtNetworkItem?.itemModel {
            #expect(listItem.subtitle == "Ethereum")
        } else {
            Issue.record("Expected network item model for generic USDT")
        }
    }

    @Test
    func networkFeeItemModel() {
        let model = ConfirmTransferSceneViewModel.mock()

        model.state = .mock(transaction: .error(AnyError("test")))
        let errorFeeItem = model.itemModel(for: .networkFee) as? ConfirmNetworkFeeViewModel

        if case let .networkFee(listItem, selectable) = errorFeeItem?.itemModel {
            #expect(listItem.subtitle == "-")
            #expect(listItem.subtitleExtra == nil)
            #expect(selectable == false)
        } else {
            Issue.record("Expected network fee item model for error state")
        }

        model.state = .mock(transaction: .data(.mock()))
        let loadedFeeItem = model.itemModel(for: .networkFee) as? ConfirmNetworkFeeViewModel

        if case let .networkFee(listItem, selectable) = loadedFeeItem?.itemModel {
            #expect(listItem.subtitle != nil)
            #expect(selectable)
        } else {
            Issue.record("Expected network fee item model with loaded fee")
        }
    }

    @Test
    func networkFeeStaysSelectableWhileReloading() {
        let model = ConfirmTransferSceneViewModel.mock()

        model.state = .mock(transaction: .loading, load: .mock(preload: .mock(confirmData: .mock(feeRates: [
            GemFeeRate(priority: .normal, gasPriceType: .regular(gasPrice: 20)),
            GemFeeRate(priority: .fast, gasPriceType: .regular(gasPrice: 30)),
        ]))))
        let reloadingFeeItem = model.itemModel(for: .networkFee) as? ConfirmNetworkFeeViewModel

        if case let .networkFee(listItem, selectable) = reloadingFeeItem?.itemModel {
            #expect(listItem.subtitle == nil)
            #expect(listItem.hasSubtitlePlaceholder)
            #expect(selectable)
        } else {
            Issue.record("Expected network fee item model while reloading")
        }
    }

    @Test
    func fetchAfterFeeChangeReplacesTheSceneWithTheServiceAnswer() async {
        let priorities: [Gemstone.FeePriority] = [.normal, .fast]
        let model = ConfirmTransferSceneViewModel.mock(
            load: .success(.mock(preload: .mock(confirmData: .mock(feeRates: [
                GemFeeRate(priority: .normal, gasPriceType: .regular(gasPrice: 20)),
                GemFeeRate(priority: .fast, gasPriceType: .regular(gasPrice: 30)),
            ])))),
        )

        await model.load()
        #expect(model.state.transaction.value?.confirmData.feeRates.map(\.priority) == priorities)

        model.state.simulation = .mock(warnings: [SimulationWarning(severity: .warning, warning: .externallyOwnedSpender, message: nil)])
        model.feeSelection = .priority(priority: .fast)
        await model.load()

        #expect(model.state.simulation.warnings.isEmpty)
        #expect(model.state.transaction.value?.confirmData.feeRates.map(\.priority) == priorities)
    }

    @Test
    func fetchIgnoresErrorAfterCancellation() async {
        let model = ConfirmTransferSceneViewModel.mock(
            load: .failure(AnyError("network")),
        )

        let task = Task { await model.load() }
        task.cancel()
        await task.value

        #expect(model.state.transaction.isLoading)
    }

    @Test
    func memoItemModel() {
        let modelWithMemo = ConfirmTransferSceneViewModel.mock(
            data: .mock(
                type: .transfer(.mock(id: .mockSolana())),
                recipient: .mock(memo: "Test memo"),
            ),
        )
        let memoItem = modelWithMemo.itemModel(for: .memo) as? ConfirmMemoViewModel

        if case let .memo(listItem) = memoItem?.itemModel {
            #expect(listItem.title == Localized.Transfer.memo)
            #expect(listItem.subtitle == "Test memo")
        } else {
            Issue.record("Expected memo item model")
        }

        let modelNoMemo = ConfirmTransferSceneViewModel.mock(
            data: .mock(type: .transfer(.mockEthereum())),
        )
        let noMemoItem = modelNoMemo.itemModel(for: .memo) as? ConfirmMemoViewModel

        if case .empty = noMemoItem?.itemModel {
            // Expected empty for non-memo chain
        } else {
            Issue.record("Expected empty for non-memo chain")
        }
    }

    @Test
    func swapDetailsItemModel() {
        let swapModel = ConfirmTransferSceneViewModel.mock(
            data: .mock(type: .swap(.mockEthereum(), .mockEthereumUSDT(), .mock())),
        )
        let swapItem = swapModel.itemModel(for: .details) as? ConfirmDetailsViewModel

        if case .swapDetails = swapItem?.itemModel {
            // Expected swap details
        } else {
            Issue.record("Expected swap details item model")
        }

        let transferModel = ConfirmTransferSceneViewModel.mock(
            data: .mock(type: .transfer(.mock())),
        )
        let transferSwapItem = transferModel.itemModel(for: .details) as? ConfirmDetailsViewModel

        if case .empty = transferSwapItem?.itemModel {
            // Expected empty for non-swap
        } else {
            Issue.record("Expected empty for non-swap transaction")
        }
    }

    @Test
    func errorItemModel() {
        let model = ConfirmTransferSceneViewModel.mock()
        model.state = .mock(transaction: .error(AnyError("Test error")))

        let errorItem = model.itemModel(for: .error) as? ConfirmErrorViewModel

        if case let .error(title, _, _) = errorItem?.itemModel {
            #expect(title == Localized.Errors.errorOccurred)
        } else if case .empty = errorItem?.itemModel {
            // Can be empty when no error
        } else {
            Issue.record("Expected error or empty item model")
        }
    }

    @Test
    func sectionsStructure() {
        let model = ConfirmTransferSceneViewModel.mock()
        let sections = model.sections

        #expect(sections.count == 4)
        #expect(sections[0].id == "header")
        #expect(sections[1].id == "details")
        #expect(sections[2].id == "fee")
        #expect(sections[3].id == "error")

        #expect(sections[0].values == [.header])
        #expect(sections[1].values == [.app, .sender, .recipient, .network, .memo, .details])
        #expect(sections[2].values == [.networkFee])
        #expect(sections[3].values == [.error])
    }

    @Test
    func walletConnectSectionsStructure() async {
        let payload = [
            SimulationPayloadField.standard(kind: .contract, value: "0x1111111111111111111111111111111111111111", fieldType: .address, display: .primary),
            SimulationPayloadField.standard(kind: .method, value: "Approve", fieldType: .text, display: .primary),
        ]
        let model = ConfirmTransferSceneViewModel.mock(
            data: .mock(type: .generic(asset: .mockEthereum(), metadata: .mock(), extra: .mock(to: "0x1111111111111111111111111111111111111111"))),
            simulation: .mock(
                warnings: [SimulationWarning(
                    severity: .warning,
                    warning: .tokenApproval(SimulationWarningApproval(assetId: AssetId(chain: .ethereum, tokenId: "0x1111111111111111111111111111111111111111"), value: "1000")),
                    message: nil,
                )],
                payload: payload,
            ),
            load: .success(.mock(
                simulation: GemConfirmSimulation(primaryFields: payload.map { $0.map() }, secondaryFields: [], header: nil, balanceChanges: [], hasCriticalWarning: false),
                warnings: [SimulationWarning(
                    severity: .warning,
                    warning: .tokenApproval(SimulationWarningApproval(assetId: AssetId(chain: .ethereum, tokenId: "0x1111111111111111111111111111111111111111"), value: "1000")),
                    message: nil,
                )],
            )),
        )
        await model.load()
        let sections = model.sections

        #expect(sections.count == 6)
        #expect(sections[0].id == "header")
        #expect(sections[1].id == "details")
        #expect(sections[2].id == "warnings")
        #expect(sections[3].id == "payload")
        #expect(sections[4].id == "fee")
        #expect(sections[5].id == "error")

        #expect(sections[1].values == [.app, .sender, .network])
        #expect(sections[2].values == [.warnings])
        #expect(sections[3].values == [.payload])
    }

    @Test
    func buttonDisabledWithCriticalWarnings() async {
        let model = ConfirmTransferSceneViewModel.mock(
            simulation: .mock(warnings: [SimulationWarning(severity: .critical, warning: .suspiciousSpender, message: nil)]),
            load: .success(.mock(
                simulation: GemConfirmSimulation(primaryFields: [], secondaryFields: [], header: nil, balanceChanges: [], hasCriticalWarning: true),
            )),
        )
        await model.load()

        #expect(model.isButtonDisabled)
    }

    @Test
    func buttonEnabledWithNoWarnings() {
        #expect(!ConfirmTransferSceneViewModel.mock().isButtonDisabled)
    }

    @Test
    func simulationWarningsPassThroughExternallyOwnedSpenderWarnings() {
        let model = ConfirmTransferSceneViewModel.mock(
            simulation: .mock(warnings: [
                SimulationWarning(
                    severity: .warning,
                    warning: .permitApproval(SimulationWarningApproval(assetId: AssetId(chain: .ethereum, tokenId: "0x123"), value: "1000")),
                    message: nil,
                ),
                SimulationWarning(
                    severity: .warning,
                    warning: .externallyOwnedSpender,
                    message: nil,
                ),
            ]),
        )

        #expect(model.simulationWarnings.count == 2)
        #expect(model.simulationWarnings.last?.warning == .externallyOwnedSpender)
        #expect(!model.isButtonDisabled)
    }

    @Test
    func simulationWarningsPassThroughValidationWarnings() {
        let model = ConfirmTransferSceneViewModel.mock(
            simulation: .mock(warnings: [
                SimulationWarning(
                    severity: .warning,
                    warning: .permitApproval(SimulationWarningApproval(assetId: AssetId(chain: .ethereum, tokenId: "0x123"), value: "1000")),
                    message: nil,
                ),
                SimulationWarning(
                    severity: .critical,
                    warning: .validationError,
                    message: "Unable to verify spender is a contract",
                ),
            ]),
        )

        #expect(model.simulationWarnings.count == 2)
        #expect(model.simulationWarnings.last?.warning == .validationError)
    }

    @Test
    func scanTransactionMaliciousError() {
        let model = ConfirmTransferSceneViewModel.mock()
        model.onSelectListError(error: .confirm(.ScanMalicious))

        guard case .info(.maliciousTransaction) = model.isPresentingSheet else {
            Issue.record("Expected maliciousTransaction sheet")
            return
        }
    }

    @Test
    func scanTransactionMemoRequiredError() {
        let model = ConfirmTransferSceneViewModel.mock()
        model.onSelectListError(error: .confirm(.ScanMemoRequired(symbol: "BTC")))

        guard case let .info(.memoRequired(symbol)) = model.isPresentingSheet else {
            Issue.record("Expected memoRequired sheet")
            return
        }
        #expect(symbol == "BTC")
    }

    @Test
    func insufficientNetworkFeeErrorShowsRequiredAmount() {
        let model = ConfirmTransferSceneViewModel.mock()
        let required = BigInt(21_000_000_000_000)
        model.onSelectListError(error: .confirm(.InsufficientNetworkFee(asset: Asset.mockEthereum().map(), requirement: GemBalanceRequirement(required: required, available: 0, shortfall: required))))

        guard case let .info(.insufficientNetworkFee(_, _, sheetRequirement, _, _, _)) = model.isPresentingSheet else {
            Issue.record("Expected insufficientNetworkFee sheet")
            return
        }
        #expect(sheetRequirement == BalanceRequirement(required: required, available: .zero, shortfall: required))
    }

    @Test
    func insufficientNetworkFeeBuyActionUsesSmallDefaultAmount() {
        let model = ConfirmTransferSceneViewModel.mock()
        model.onSelectListError(error: .confirm(.InsufficientNetworkFee(asset: Asset.mockEthereum().map(), requirement: nil)))

        guard case let .info(.insufficientNetworkFee(_, _, _, _, _, .action(_, action))) = model.isPresentingSheet else {
            Issue.record("Expected insufficientNetworkFee sheet")
            return
        }

        action()

        guard case let .fiatConnect(_, _, amount) = model.isPresentingSheet else {
            Issue.record("Expected fiatConnect sheet")
            return
        }
        #expect(amount == FiatConfig.insufficientNetworkFeeBuyAmount)
    }

    @Test
    func swapFromAssetUsesLoadedFeeAsset() {
        let asset = Asset.mockTempoPathUSD()
        let feeAsset = Asset.mockTempoUSDC()
        let model = ConfirmTransferSceneViewModel.mock(data: .mock(type: .transfer(asset)))
        model.state = .mock(transaction: .data(.mock(feeAsset: feeAsset)), feeAsset: feeAsset)

        #expect(model.swapFromAsset(to: asset) == feeAsset)
    }

    @Test
    func tronInsufficientBalanceActionShowsGetOptions() {
        let model = ConfirmTransferSceneViewModel.mock(data: .mock(type: .transfer(.mockTronUSDT())))
        model.onSelectListError(error: .confirm(.InsufficientBalance(asset: Asset.mockTron().map(), requirement: GemBalanceRequirement(required: 36_798_300, available: 36_070_000, shortfall: 728_300))))

        guard case let .info(sheet) = model.isPresentingSheet,
              case let .balanceRequired(_, _, requirement, .action(_, action)) = sheet
        else {
            Issue.record("Expected balanceRequired sheet")
            return
        }
        #expect(requirement == BalanceRequirement(required: 36_798_300, available: 36_070_000, shortfall: 728_300))
        #expect(InfoSheetModelFactory.create(from: sheet).description == Localized.Info.balanceRequiredDescription(
            "36.7983 TRX".boldMarkdown(),
            "36.07 TRX".boldMarkdown(),
            "0.7283 TRX".boldMarkdown(),
        ))

        action()

        guard case let .getAsset(asset, buyAmount) = model.isPresentingSheet else {
            Issue.record("Expected getAsset sheet")
            return
        }
        #expect(asset.id == Asset.mockTron().id)
        #expect(buyAmount == nil)
    }

    @Test
    func tronTokenInsufficientBalancePreservesAsset() {
        let asset = Asset.mockTronUSDT()
        let model = ConfirmTransferSceneViewModel.mock(data: .mock(type: .transfer(asset)))
        model.onSelectListError(error: .confirm(.InsufficientBalance(asset: asset.map(), requirement: GemBalanceRequirement(required: 2, available: 1, shortfall: 1))))

        guard case let .info(.balanceRequired(_, _, _, .action(_, action))) = model.isPresentingSheet else {
            Issue.record("Expected balanceRequired sheet")
            return
        }
        action()

        guard case let .getAsset(sheetAsset, buyAmount) = model.isPresentingSheet else {
            Issue.record("Expected getAsset sheet")
            return
        }
        #expect(sheetAsset.id == asset.id)
        #expect(model.assetAddress(sheetAsset).asset.id == asset.id)
        #expect(buyAmount == nil)
    }

    @Test
    func insufficientBalanceBuyActionUsesErrorAsset() {
        let asset = Asset.mockEthereumUSDT()
        let model = ConfirmTransferSceneViewModel.mock(data: .mock(type: .transfer(asset)))
        model.onSelectListError(error: .confirm(.InsufficientBalance(asset: asset.map(), requirement: GemBalanceRequirement(required: 2, available: 1, shortfall: 1))))

        guard case let .info(.balanceRequired(_, _, _, .action(_, action))) = model.isPresentingSheet else {
            Issue.record("Expected balanceRequired sheet")
            return
        }
        action()

        guard case let .fiatConnect(assetAddress, _, amount) = model.isPresentingSheet else {
            Issue.record("Expected fiatConnect sheet")
            return
        }
        #expect(assetAddress.asset.id == asset.id)
        #expect(amount == nil)
    }

    @Test
    func insufficientNetworkFeeSheetShowsRequiredFeeWithFiat() {
        let asset = Asset.mockEthereum()
        let feeAsset = asset.chain.asset
        let image = AssetViewModel(asset: asset).assetImage
        let required = BigInt(2_000_000_000_000_000)

        let withPrice = InfoSheetModelFactory.create(from: .insufficientNetworkFee(
            asset, image: image, requirement: BalanceRequirement(required: required, available: .zero, shortfall: required),
            price: Price(price: 2000, priceChangePercentage24h: 0, updatedAt: Date()),
            currency: "USD", button: .action(title: "", action: {}),
        ))
        let withoutPrice = InfoSheetModelFactory.create(from: .insufficientNetworkFee(
            asset, image: image, requirement: BalanceRequirement(required: required, available: .zero, shortfall: required),
            price: nil, currency: "USD", button: .action(title: "", action: {}),
        ))

        #expect(withPrice.description == Localized.Info.InsufficientNetworkFeeBalance.description(
            "0.002 ETH (~$4.00)".boldMarkdown(),
            feeAsset.name.boldMarkdown(),
            "0 ETH".boldMarkdown(),
            "0.002 ETH".boldMarkdown(),
        ))
        #expect(withoutPrice.description == Localized.Info.InsufficientNetworkFeeBalance.description(
            "0.002 ETH".boldMarkdown(),
            feeAsset.name.boldMarkdown(),
            "0 ETH".boldMarkdown(),
            "0.002 ETH".boldMarkdown(),
        ))
    }

    @Test
    func tronInsufficientNetworkFeeUsesFeeAsset() {
        let model = ConfirmTransferSceneViewModel.mock(data: .mock(type: .transfer(.mockTronUSDT())))
        model.onSelectListError(error: .confirm(.InsufficientNetworkFee(asset: Asset.mockTron().map(), requirement: nil)))

        guard case let .info(sheet) = model.isPresentingSheet,
              case let .insufficientNetworkFee(asset, _, _, _, _, .action(_, action)) = sheet
        else {
            Issue.record("Expected insufficientNetworkFee sheet")
            return
        }
        #expect(asset.id == Asset.mockTron().id)
        #expect(InfoSheetModelFactory.create(from: sheet).buttonTitle == Localized.Asset.getAsset("TRX"))

        action()

        guard case let .getAsset(asset, buyAmount) = model.isPresentingSheet else {
            Issue.record("Expected getAsset sheet")
            return
        }
        #expect(asset.id == Asset.mockTron().id)
        #expect(buyAmount == FiatConfig.insufficientNetworkFeeBuyAmount)
    }

    private func verifyNonEmpty(_ model: any ItemModelProvidable<ConfirmTransferItemModel>) {
        if case .empty = model.itemModel {
            Issue.record("Expected non-empty model")
        }
    }
}
