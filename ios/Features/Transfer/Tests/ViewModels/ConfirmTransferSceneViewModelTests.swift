// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Foundation
import enum Gemstone.FeePriority
import class Gemstone.GemAssetConfigService
import struct Gemstone.GemBalanceRequirement
import enum Gemstone.GemConfirmRowContent
import enum Gemstone.GemConfirmError
import struct Gemstone.GemConfirmFailure
import struct Gemstone.GemFeeRate
import struct Gemstone.GemSimulationWarningRow
import protocol Gemstone.GemNameServiceProtocol
import struct Gemstone.GemTransferData
import struct Gemstone.PaymentInvoice
import struct Gemstone.PaymentQuote
import struct Gemstone.PaymentVerification
import func Gemstone.walletRow
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
@testable import TransferTestKit
import struct Gemstone.SimulationPayloadField

@MainActor
struct ConfirmTransferSceneViewModelTests {
    @Test
    func paymentHeaderAppearsAfterLoading() {
        let data = GemTransferData.mockPayment()
        let model = ConfirmTransferSceneViewModel.mock(data: data)

        #expect(model.isHeaderVisible == false)
        model.state.load = .mock(preload: .mock())
        #expect(model.isHeaderVisible == true)
    }

    @Test
    func paymentHeaderShowsWhenAmountIsKnown() {
        let model = ConfirmTransferSceneViewModel.mock(data: .mockPayment(value: 1))

        #expect(model.isHeaderVisible == true)
    }

    @Test
    func selectingAnotherPaymentAssetReloadsWithIt() async {
        let invoice = PaymentInvoice.mock(quotes: [.mock(asset: .mockEthereum()), .mock(asset: .mockBNB())])
        let bnb = GemTransferData.mockPayment(asset: .mockBNB(), invoice: invoice)
        let confirmation = GemConfirmationMock(state: .mock(preload: nil), load: .success(.mock(transfer: bnb)))
        let model = ConfirmTransferSceneViewModel.mock(data: .mockPayment(asset: .mockEthereum(), invoice: invoice), confirmation: confirmation)
        model.state.screen = .mock(phase: .ready)
        model.onSelectPaymentAsset()
        guard case let .paymentAsset(selection)? = model.isPresentingSheet else {
            Issue.record("Expected the asset picker")
            return
        }
        #expect(selection == .payment([Asset.mockEthereum().id, Asset.mockBNB().id]))

        model.selectPaymentAsset(.mockBNB())
        await model.load()

        #expect(model.isPresentingSheet == nil)
        #expect(confirmation.loadOptions.last?.assetId == Asset.mockBNB().id.identifier)
        #expect(model.transfer.chain == .smartChain)
        #expect(model.state.preload != nil)
    }

    @Test
    func gatedPaymentAssetReplacesTheFeeRowAndOpensTheForm() async {
        let invoice = PaymentInvoice.mock(quotes: [.mock(asset: .mockEthereum()), .mock(asset: .mockBNB())], verification: PaymentVerification(url: "https://walletconnect.com/collect"))
        let gated = GemTransferData.mockPayment(asset: .mockBNB(), invoice: invoice)
        let confirmation = GemConfirmationMock(state: .mock(preload: nil), load: .success(.mock(transfer: gated, preload: nil)))
        let model = ConfirmTransferSceneViewModel.mock(data: .mockPayment(asset: .mockEthereum(), invoice: .mock()), confirmation: confirmation)

        model.selectPaymentAsset(.mockBNB())
        await model.load()

        #expect(model.transfer.chain == .smartChain)
        #expect(model.sections.contains { $0.values.contains(.verification) })
        #expect(model.button.state == .disabled)

        model.onSelectVerification()

        guard case let .paymentVerification(url)? = model.isPresentingSheet else {
            Issue.record("Expected the verification sheet")
            return
        }
        #expect(url.absoluteString == "https://walletconnect.com/collect")
    }

    @Test
    func failedPaymentAssetSwitchShowsTheErrorOnTheAssetItBelongsTo() async {
        let invoice = PaymentInvoice.mock(quotes: [.mock(asset: .mockBNB()), .mock(asset: .mockEthereum())])
        let picked = GemTransferData.mockPayment(asset: .mockEthereum(), invoice: invoice)
        let confirmation = GemConfirmationMock(state: .mock(transfer: picked, preload: nil), load: .failure(AnyError("gateway")))
        let model = ConfirmTransferSceneViewModel.mock(data: .mockPayment(asset: .mockBNB(), invoice: invoice), confirmation: confirmation)

        model.selectPaymentAsset(.mockEthereum())
        await model.load()

        #expect(model.transfer.chain == .ethereum, "the header follows the asset the load failed for")
        #expect(model.state.load != nil, "a failed load keeps what the screen already showed")
        #expect(model.state.transactionError != nil)
    }

    @Test
    func selectingTheSamePaymentAssetOnlyClosesTheSheet() async {
        let invoice = PaymentInvoice.mock(quotes: [.mock(asset: .mockBNB())])
        let model = ConfirmTransferSceneViewModel.mock(data: .mockPayment(asset: .mockBNB(), invoice: invoice))
        await model.load()
        model.onSelectPaymentAsset()

        model.selectPaymentAsset(.mockBNB())

        #expect(model.assetSelection == nil)
        #expect(model.isPresentingSheet == nil)
        #expect(model.state.preload != nil)
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
        let appItem = model.itemModel(for: .app) as? ConfirmRowViewModel

        if case .empty = appItem?.itemModel {
            // Expected empty for non-generic transfer
        } else {
            Issue.record("Expected empty app item model")
        }

        let modelWithWebsite = ConfirmTransferSceneViewModel.mock(rows: { _ in [.app(name: "Gem Wallet", iconUrl: nil)] })
        let appItemWithWebsite = modelWithWebsite.itemModel(for: .app) as? ConfirmRowViewModel

        if case let .app(listItem) = appItemWithWebsite?.itemModel {
            #expect(listItem.subtitle == "Gem Wallet")
        } else {
            Issue.record("Expected app item model")
        }
    }

    @Test
    func title() {
        #expect(ConfirmTransferSceneViewModel.mock(data: .mock(type: .transfer(.mock()))).title == Localized.Transfer.Send.title)
        #expect(ConfirmTransferSceneViewModel.mock(data: .mock(type: .swap(.mock(), .mock(), .mock()))).title == Localized.Wallet.swap)
        #expect(ConfirmTransferSceneViewModel.mock(data: .mock(type: .tokenApprove(.mock(), .mock()))).title == Localized.Transfer.Approve.title)
        #expect(ConfirmTransferSceneViewModel.mock(data: .mock(type: .generic(asset: .mock(), metadata: .mock(), extra: .mock()))).title == Localized.Transfer.reviewRequest)
    }

    @Test
    func senderItemModel() {
        let model = ConfirmTransferSceneViewModel.mock()
        let senderItem = model.itemModel(for: .sender) as? ConfirmRowViewModel

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
        let recipientItem = model.itemModel(for: .recipient) as? ConfirmRowViewModel

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
        let recipientItem = model.itemModel(for: .recipient) as? ConfirmRowViewModel

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
        let recipientItem = model.itemModel(for: .recipient) as? ConfirmRowViewModel

        if case let .recipient(addressViewModel) = recipientItem?.itemModel {
            #expect(addressViewModel.account.address == checksummedAddress)
            #expect(addressViewModel.account.name == "Uniswap")
        } else {
            Issue.record("Expected recipient item model")
        }
    }

    @Test
    func networkItemModel() {
        let model = ConfirmTransferSceneViewModel.mock(rows: { _ in [.network(chain: Chain.ethereum.rawValue, name: "Ethereum (ERC20)")] })
        let networkItem = model.itemModel(for: .network) as? ConfirmRowViewModel

        if case let .network(listItem) = networkItem?.itemModel {
            #expect(listItem.subtitle == "Ethereum (ERC20)")
        } else {
            Issue.record("Expected network item model")
        }
    }

    @Test
    func networkFeeItemModel() {
        let model = ConfirmTransferSceneViewModel.mock()

        model.state = .mock(screen: .mock(phase: .failed, failure: GemConfirmFailure(stage: .load, error: .Load(msg: "test"))))
        let errorFeeItem = model.itemModel(for: .networkFee) as? ConfirmNetworkFeeViewModel

        if case let .networkFee(listItem, selectable) = errorFeeItem?.itemModel {
            #expect(listItem.subtitle == "-")
            #expect(listItem.subtitleExtra == nil)
            #expect(selectable == false)
        } else {
            Issue.record("Expected network fee item model for error state")
        }

        model.state = .mock(load: .mock(preload: .mock()), screen: .mock(phase: .ready))
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

        model.state = .mock(load: .mock(preload: .mock(confirmData: .mock(feeRates: [
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
        #expect(model.state.preload?.confirmData.feeRates.map(\.priority) == priorities)

        model.state.simulation = .mock(warnings: [.mock(kind: .externallyOwnedSpender)])
        model.feeSelection = .priority(priority: .fast)
        await model.load()

        #expect(model.state.simulation.warnings.isEmpty)
        #expect(model.state.preload?.confirmData.feeRates.map(\.priority) == priorities)
    }

    @Test
    func reloadKeepsTheLoadedFeeRowUntilTheConfirmationAnswers() async {
        let confirmationMock = GemConfirmationMock(
            state: .mock(preload: nil),
            load: .success(.mock(preload: .mock(confirmData: .mock(feeRates: [
                GemFeeRate(priority: .normal, gasPriceType: .regular(gasPrice: 20)),
                GemFeeRate(priority: .fast, gasPriceType: .regular(gasPrice: 30)),
            ])))),
        )
        let model = ConfirmTransferSceneViewModel.mock(confirmation: confirmationMock)
        await model.load()

        await confirmation { reloading in
            confirmationMock.onLoad = {
                let feeItem = model.itemModel(for: .networkFee) as? ConfirmNetworkFeeViewModel
                guard case let .networkFee(listItem, selectable) = feeItem?.itemModel else { return }
                #expect(model.state.screen.phase == .loading)
                #expect(model.state.confirmData != nil)
                #expect(listItem.hasSubtitlePlaceholder)
                #expect(selectable)
                reloading()
            }
            model.feeSelection = .priority(priority: .fast)
            await model.load()
        }
        #expect(model.state.preload?.confirmData.feeRates.count == 2)
    }

    @Test
    func firstLoadShowsTheScreenBeforeThePreloadArrives() async {
        let confirmationMock = GemConfirmationMock(
            state: .mock(addressName: .mock(name: "vitalik.eth"), preload: nil),
            load: .success(.mock(addressName: .mock(name: "vitalik.eth"))),
        )
        let model = ConfirmTransferSceneViewModel.mock(confirmation: confirmationMock)

        await confirmation { preloading in
            confirmationMock.onLoad = {
                #expect(model.state.screen.phase == .loading)
                #expect(model.state.addressName?.name == "vitalik.eth")
                preloading()
            }
            await model.load()
        }
        #expect(model.state.preload != nil)
        #expect(model.state.addressName?.name == "vitalik.eth")
    }

    @Test
    func fetchIgnoresErrorAfterCancellation() async {
        let model = ConfirmTransferSceneViewModel.mock(
            load: .failure(AnyError("network")),
        )

        let task = Task { await model.load() }
        task.cancel()
        await task.value

        #expect(model.state.screen.phase == .loading)
    }

    @Test
    func memoItemModel() {
        let modelWithMemo = ConfirmTransferSceneViewModel.mock(
            data: .mock(
                type: .transfer(.mock(id: .mockSolana())),
                recipient: .mock(memo: "Test memo"),
            ),
        )
        let memoItem = modelWithMemo.itemModel(for: .memo) as? ConfirmRowViewModel

        if case let .memo(listItem) = memoItem?.itemModel {
            #expect(listItem.title == Localized.Transfer.memo)
            #expect(listItem.subtitle == "Test memo")
        } else {
            Issue.record("Expected memo item model")
        }

        let modelNoMemo = ConfirmTransferSceneViewModel.mock(
            data: .mock(type: .transfer(.mockEthereum())),
        )
        let noMemoItem = modelNoMemo.itemModel(for: .memo) as? ConfirmRowViewModel

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
        model.state = .mock(screen: .mock(phase: .failed, failure: GemConfirmFailure(stage: .load, error: .Load(msg: "Test error"))))

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
    func missingWalletDataErrorDetails() {
        for (error, description) in [
            (GemConfirmError.BalanceMissing(assetId: "tron"), "no stored balance for tron"),
            (GemConfirmError.AccountMissing(chain: Primitives.Chain.tron.rawValue), Localized.Errors.walletAccountMissing),
        ] {
            let model = ConfirmTransferSceneViewModel.mock()
            model.state = .mock(screen: .mock(phase: .failed, failure: GemConfirmFailure(stage: .load, error: error)))

            let errorItem = model.itemModel(for: .error) as? ConfirmErrorViewModel
            guard case let .error(_, displayError, _) = errorItem?.itemModel else {
                Issue.record("Expected wallet data error item")
                continue
            }
            #expect(displayError.localizedDescription == description)
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
        #expect(sections[1].values == [.sender, .recipient, .network, .details], "a send on a chain without memos has no app or memo row")
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
                warnings: [.mock(warning: .tokenApproval(.mock(assetId: AssetId(chain: .ethereum, tokenId: "0x1111111111111111111111111111111111111111"))))],
                payload: payload,
            ),
            load: .success(.mock(
                simulation: .mock(primaryFields: payload),
                warnings: [.mock()],
            )),
            rows: { _ in
                [
                    .app(name: "Gem Wallet", iconUrl: nil),
                    .sender(wallet: walletRow(wallet: Wallet.mock().toGem())),
                    .network(chain: Chain.ethereum.rawValue, name: "Ethereum"),
                ]
            },
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
            simulation: .mock(warnings: [.mock(severity: .critical, warning: .suspiciousSpender)]),
            load: .success(.mock(
                simulation: .mock(hasCriticalWarning: true),
            )),
        )
        await model.load()

        #expect(model.button.state == .disabled)
    }

    @Test
    func buttonEnabledWithNoWarnings() {
        #expect(ConfirmTransferSceneViewModel.mock().button.state == .loading)
    }

    @Test
    func simulationWarningsHideBoundedApprovalsAndKeepExternallyOwnedSpenderWarnings() {
        let model = ConfirmTransferSceneViewModel.mock(
            simulation: .mock(warnings: [
                .mock(warning: .permitApproval(.mock(value: 1000))),
                .mock(warning: .externallyOwnedSpender),
            ]),
        )

        #expect(model.simulationWarnings.map(\.kind) == [.externallyOwnedSpender])
        #expect(model.button.state != .disabled)
    }

    @Test
    func simulationWarningsPassThroughValidationWarnings() {
        let model = ConfirmTransferSceneViewModel.mock(
            simulation: .mock(warnings: [
                .mock(warning: .permitApproval(.mock(value: 1000))),
                .mock(severity: .critical, warning: .validationError, message: "Unable to verify spender is a contract"),
            ]),
        )

        #expect(model.simulationWarnings.map(\.kind) == [.validationError])
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
        model.onSelectListError(error: .confirm(.InsufficientNetworkFee(asset: Asset.mockEthereum().toGem(), requirement: GemBalanceRequirement(required: required, available: 0, shortfall: required))))

        guard case let .info(.insufficientNetworkFee(_, _, sheetRequirement, _, _, _)) = model.isPresentingSheet else {
            Issue.record("Expected insufficientNetworkFee sheet")
            return
        }
        #expect(sheetRequirement == BalanceRequirement(required: required, available: .zero, shortfall: required))
    }

    @Test
    func insufficientNetworkFeeBuyActionUsesSmallDefaultAmount() {
        let model = ConfirmTransferSceneViewModel.mock()
        model.onSelectListError(error: .confirm(.InsufficientNetworkFee(asset: Asset.mockEthereum().toGem(), requirement: nil)))

        guard case let .info(.insufficientNetworkFee(_, _, _, _, _, .action(_, action))) = model.isPresentingSheet else {
            Issue.record("Expected insufficientNetworkFee sheet")
            return
        }

        action()

        guard case let .fiatConnect(_, _, amount) = model.isPresentingSheet else {
            Issue.record("Expected fiatConnect sheet")
            return
        }
        #expect(amount == Int(GemConfirmationMock.networkFeeBuyAmount))
    }

    @Test
    func swapFromAssetUsesLoadedFeeAsset() {
        let asset = Asset.mockTempoPathUSD()
        let feeAsset = Asset.mockTempoUSDC()
        let data = GemTransferData.mock(type: .transfer(asset))
        let model = ConfirmTransferSceneViewModel.mock(data: data)
        model.state = .mock(load: .mock(transfer: data, preload: .mock()), feeAsset: feeAsset, screen: .mock(phase: .ready))

        #expect(model.swapFromAsset(to: asset) == feeAsset)
    }

    @Test
    func tronInsufficientBalanceActionShowsGetOptions() {
        let model = ConfirmTransferSceneViewModel.mock(data: .mock(type: .transfer(.mockTronUSDT())))
        model.onSelectListError(error: .confirm(.InsufficientBalance(asset: Asset.mockTron().toGem(), requirement: GemBalanceRequirement(required: 36_798_300, available: 36_070_000, shortfall: 728_300))))

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
        model.onSelectListError(error: .confirm(.InsufficientBalance(asset: asset.toGem(), requirement: GemBalanceRequirement(required: 2, available: 1, shortfall: 1))))

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
        model.onSelectListError(error: .confirm(.InsufficientBalance(asset: asset.toGem(), requirement: GemBalanceRequirement(required: 2, available: 1, shortfall: 1))))

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
            price: .mock(price: 2000),
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
        model.onSelectListError(error: .confirm(.InsufficientNetworkFee(asset: Asset.mockTron().toGem(), requirement: nil)))

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
        #expect(buyAmount == Int(GemConfirmationMock.networkFeeBuyAmount))
    }

    private func verifyNonEmpty(_ model: any ItemModelProvidable<ConfirmTransferItemModel>) {
        if case .empty = model.itemModel {
            Issue.record("Expected non-empty model")
        }
    }
}

