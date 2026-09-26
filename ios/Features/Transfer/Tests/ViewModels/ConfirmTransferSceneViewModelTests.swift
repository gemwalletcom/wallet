// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Foundation
import func Gemstone.addressCopy
import struct Gemstone.AssetPrice
import func Gemstone.confirmErrorInfo
import func Gemstone.feeAmount
import class Gemstone.GemAssetConfigService
import struct Gemstone.GemBalanceRequirement
import enum Gemstone.GemConfirmError
import struct Gemstone.GemConfirmFailure
import struct Gemstone.GemConfirmFee
import enum Gemstone.GemConfirmRowContent
import enum Gemstone.GemFeeRateKind
import enum Gemstone.GemListRow
import protocol Gemstone.GemNameServiceProtocol
import enum Gemstone.GemRowMenuItem
import struct Gemstone.GemSimulationPayloadRow
import struct Gemstone.GemTransferData
import struct Gemstone.PaymentInvoice
import struct Gemstone.PaymentQuote
import struct Gemstone.PaymentVerification
import struct Gemstone.SimulationPayloadField
import enum Gemstone.TransactionInputType
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

@MainActor
struct ConfirmTransferSceneViewModelTests {
    @Test
    func selectingAnotherPaymentAssetReloadsWithIt() async {
        let invoice = PaymentInvoice.mock(link: .walletConnectPay(paymentId: "pay_123"), merchant: .mock(name: "Merchant", icon: "https://example.com/icon.png"), price: .mock(currency: "USD", amount: 0.30), quotes: [
            .mock(id: "option-ethereum", assetId: "ethereum", value: 1_000_000_000_000_000),
            .mock(id: "option-smartchain", assetId: "smartchain", value: 1_000_000_000_000_000),
        ])
        let bnb = GemTransferData.mock(inputType: .payment(asset: Primitives.Asset.mock(id: .mock(chain: .smartChain), name: "BNB", symbol: "BNB", decimals: 18).toGem(), invoice: invoice, extra: .mock(data: Data("transaction".utf8))))
        let confirmation = GemConfirmationMock(
            state: .mock(fee: nil),
            load: .success(.mock(transfer: bnb, fee: .mock())),
            rows: { _ in [.paymentAsset(
                symbol: "ETH",
                selectable: true,
                assetIds: [Asset.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18).id.identifier, Asset.mock(id: .mock(chain: .smartChain), name: "BNB", symbol: "BNB", decimals: 18).id.identifier],
            )] },
        )
        let model = ConfirmTransferSceneViewModel.mock(
            data: .mock(inputType: .payment(asset: Primitives.Asset.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18).toGem(), invoice: invoice, extra: .mock(data: Data("transaction".utf8)))),
            confirmation: confirmation,
        )
        model.state.screen = .mock(phase: .ready)
        model.onSelectPaymentAsset()
        guard case let .paymentAsset(selection)? = model.isPresentingSheet else {
            Issue.record("Expected the asset picker")
            return
        }
        #expect(selection == .payment([Asset.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18).id, Asset.mock(id: .mock(chain: .smartChain), name: "BNB", symbol: "BNB", decimals: 18).id]))

        model.selectPaymentAsset(.mock(id: .mock(chain: .smartChain), name: "BNB", symbol: "BNB", decimals: 18))
        await model.load()

        #expect(model.isPresentingSheet == nil)
        #expect(confirmation.requestedOptions.last?.assetId == Asset.mock(id: .mock(chain: .smartChain), name: "BNB", symbol: "BNB", decimals: 18).id.identifier)
        #expect(model.transfer.chain == .smartChain)
        #expect(model.state.fee != nil)
    }

    @Test
    func gatedPaymentAssetReplacesTheFeeRowAndOpensTheForm() async {
        let invoice = PaymentInvoice.mock(
            link: .walletConnectPay(paymentId: "pay_123"),
            merchant: .mock(name: "Merchant", icon: "https://example.com/icon.png"),
            price: .mock(currency: "USD", amount: 0.30),
            quotes: [.mock(id: "option-ethereum", assetId: "ethereum", value: 1_000_000_000_000_000), .mock(id: "option-smartchain", assetId: "smartchain", value: 1_000_000_000_000_000)],
            verification: PaymentVerification(url: "https://walletconnect.com/collect"),
        )
        let gated = GemTransferData.mock(inputType: .payment(asset: Primitives.Asset.mock(id: .mock(chain: .smartChain), name: "BNB", symbol: "BNB", decimals: 18).toGem(), invoice: invoice, extra: .mock(data: Data("transaction".utf8))))
        let confirmation = GemConfirmationMock(state: .mock(fee: nil), load: .success(.mock(transfer: gated, fee: nil)))
        let model = ConfirmTransferSceneViewModel.mock(
            data: .mock(inputType: .payment(
                asset: Primitives.Asset.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18).toGem(),
                invoice: .mock(
                    link: .walletConnectPay(paymentId: "pay_123"),
                    merchant: .mock(name: "Merchant", icon: "https://example.com/icon.png"),
                    price: .mock(currency: "USD", amount: 0.30),
                    quotes: [.mock(id: "option-ethereum", assetId: "ethereum", value: 1_000_000_000_000_000)],
                ),
                extra: .mock(data: Data("transaction".utf8)),
            )),
            confirmation: confirmation,
        )

        model.selectPaymentAsset(.mock(id: .mock(chain: .smartChain), name: "BNB", symbol: "BNB", decimals: 18))
        await model.load()

        #expect(model.transfer.chain == .smartChain)
        #expect(model.sections.contains { $0.values.contains(.verification) })
        #expect(model.viewState.button.state == .disabled)

        model.onSelectVerification()

        guard case let .paymentVerification(url)? = model.isPresentingSheet else {
            Issue.record("Expected the verification sheet")
            return
        }
        #expect(url.absoluteString == "https://walletconnect.com/collect")
    }

    @Test
    func failedPaymentAssetSwitchShowsTheErrorOnTheAssetItBelongsTo() async {
        let invoice = PaymentInvoice.mock(link: .walletConnectPay(paymentId: "pay_123"), merchant: .mock(name: "Merchant", icon: "https://example.com/icon.png"), price: .mock(currency: "USD", amount: 0.30), quotes: [
            .mock(id: "option-smartchain", assetId: "smartchain", value: 1_000_000_000_000_000),
            .mock(id: "option-ethereum", assetId: "ethereum", value: 1_000_000_000_000_000),
        ])
        let shown = GemTransferData.mock(inputType: .payment(asset: Primitives.Asset.mock(id: .mock(chain: .smartChain), name: "BNB", symbol: "BNB", decimals: 18).toGem(), invoice: invoice, extra: .mock(data: Data("transaction".utf8))))
        let picked = GemTransferData.mock(inputType: .payment(asset: Primitives.Asset.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18).toGem(), invoice: invoice, extra: .mock(data: Data("transaction".utf8))))
        let confirmation = GemConfirmationMock(state: .mock(transfer: shown, fee: nil), load: .failure(GemConfirmError.Load(msg: "gateway")), selection: picked)
        let model = ConfirmTransferSceneViewModel.mock(data: shown, confirmation: confirmation)

        model.selectPaymentAsset(.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18))
        await model.load()

        #expect(model.transfer.chain == .ethereum, "the header follows the asset the load failed for")
        #expect(model.state.load != nil, "a failed load keeps what the screen already showed")
        #expect(model.state.loadError != nil)
    }

    @Test
    func loadSupersededByANewerLoadShowsNoError() async {
        let model = ConfirmTransferSceneViewModel.mock(load: .failure(GemConfirmError.Cancelled))

        await model.load()

        #expect(model.state.loadError == nil)
    }

    @Test
    func selectingTheSamePaymentAssetOnlyClosesTheSheet() async {
        let invoice = PaymentInvoice.mock(
            link: .walletConnectPay(paymentId: "pay_123"),
            merchant: .mock(name: "Merchant", icon: "https://example.com/icon.png"),
            price: .mock(currency: "USD", amount: 0.30),
            quotes: [.mock(id: "option-smartchain", assetId: "smartchain", value: 1_000_000_000_000_000)],
        )
        let model = ConfirmTransferSceneViewModel.mock(data: .mock(inputType: .payment(
            asset: Primitives.Asset.mock(id: .mock(chain: .smartChain), name: "BNB", symbol: "BNB", decimals: 18).toGem(),
            invoice: invoice,
            extra: .mock(data: Data("transaction".utf8)),
        )))
        await model.load()
        model.onSelectPaymentAsset()

        model.selectPaymentAsset(.mock(id: .mock(chain: .smartChain), name: "BNB", symbol: "BNB", decimals: 18))

        #expect(model.loadOptions.assetId == nil)
        #expect(model.isPresentingSheet == nil)
        #expect(model.state.fee != nil)
    }

    @Test
    func itemModelReturnsNonEmpty() {
        let model = ConfirmTransferSceneViewModel.mock()

        verifyNonEmpty(model.itemModel(for: .header))
        verifyNonEmpty(model.itemModel(for: .row(0)))
        verifyNonEmpty(model.itemModel(for: .row(1)))
        verifyNonEmpty(model.itemModel(for: .row(2)))
        verifyNonEmpty(model.itemModel(for: .networkFee))
    }

    @Test
    func headerItemModel() {
        let model = ConfirmTransferSceneViewModel.mock(
            data: .mock(inputType: .transfer(asset: Primitives.Asset.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18).toGem())),
        )
        let headerItem = model.itemModel(for: .header)

        if case .header = headerItem {
            // Expected header item
        } else {
            Issue.record("Expected header item model")
        }
    }

    @Test
    func appItemModel() {
        let website: GemRowMenuItem = .open(title: .rowTitle(title: .website), url: "https://gemwallet.com")
        let model = ConfirmTransferSceneViewModel.mock(rows: { _ in [.row(row: .app(title: .app, name: "Gem Wallet", iconUrl: nil, menu: [website]))] })
        let appItem = model.itemModel(for: .row(0))

        if case let .row(.app(_, name, _, menu)) = appItem {
            #expect(name == "Gem Wallet")
            #expect(menu == [website])
        } else {
            Issue.record("Expected app row")
        }
    }

    @Test
    func title() {
        #expect(ConfirmTransferSceneViewModel.mock(data: .mock(inputType: .transfer(asset: Primitives.Asset.mock().toGem()))).title == Localized.Transfer.Send.title)
        #expect(ConfirmTransferSceneViewModel.mock(data: .mock(inputType: .swap(fromAsset: Primitives.Asset.mock().toGem(), toAsset: Primitives.Asset.mock().toGem(), swapData: .mock()))).title == Localized.Wallet.swap)
        #expect(ConfirmTransferSceneViewModel.mock(data: .mock(inputType: .tokenApprove(asset: Primitives.Asset.mock().toGem(), approvalData: .mock()))).title == Localized.Transfer.Approve.title)
        #expect(ConfirmTransferSceneViewModel.mock(data: .mock(inputType: .generic(asset: Primitives.Asset.mock().toGem(), metadata: Primitives.ApplicationMetadata.mock().toGem(), extra: .mock()))).title == Localized.Transfer.reviewRequest)
    }

    @Test
    func senderItemModel() {
        let model = ConfirmTransferSceneViewModel.mock()
        let senderItem = model.itemModel(for: .row(0))

        let expected = Wallet.mock(accounts: [.mock(chain: GemTransferData.mock().chain)])
        if case let .row(.wallet(_, wallet, menu)) = senderItem, case let .copy(copy) = menu.first {
            #expect(wallet.name == expected.name)
            #expect(copy.value == expected.accounts[0].address)
        } else {
            Issue.record("Expected wallet row")
        }
    }

    @Test
    func recipientItemModel() {
        let address = "0x1234567890123456789012345678901234567890"
        let model = ConfirmTransferSceneViewModel.mock(data: .mock(
            inputType: .transfer(asset: Primitives.Asset.mock().toGem()),
            recipient: .mock(address: address),
        ))
        let recipientItem = model.itemModel(for: .row(1))

        if case let .recipient(row) = recipientItem {
            #expect(row.address == address)
            #expect(row.text.text != address, "an unnamed address reads short")
        } else {
            Issue.record("Expected recipient item model")
        }
    }

    @Test
    func recipientNameItemModel() async {
        let address = "bc1qml9s2f9k8wc0882x63lyplzp97srzg2c39fyaw"
        let model = ConfirmTransferSceneViewModel.mock(
            data: .mock(
                inputType: .transfer(asset: Primitives.Asset.mock().toGem()),
                recipient: .mock(address: address),
            ),
            load: .success(.mock(addressName: Primitives.AddressName.mock(chain: .bitcoin, address: address, name: "Bitcoin").toGem())),
        )
        await model.load()
        let recipientItem = model.itemModel(for: .row(1))

        if case let .recipient(row) = recipientItem {
            #expect(row.address == address)
            #expect(row.text.text == "Bitcoin")
        } else {
            Issue.record("Expected recipient item model")
        }
    }

    @Test
    func recipientNameItemModelUsesStoredAddress() async {
        let checksummedAddress = "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4"
        let model = ConfirmTransferSceneViewModel.mock(
            data: .mock(
                inputType: .transfer(asset: Primitives.Asset.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18).toGem()),
                recipient: .mock(address: checksummedAddress),
            ),
            load: .success(.mock(addressName: Primitives.AddressName.mock(chain: .ethereum, address: checksummedAddress, name: "Uniswap").toGem())),
        )
        await model.load()
        let recipientItem = model.itemModel(for: .row(1))

        if case let .recipient(row) = recipientItem {
            #expect(row.address == checksummedAddress)
            #expect(row.text.text == "Uniswap")
        } else {
            Issue.record("Expected recipient item model")
        }
    }

    @Test
    func networkItemModel() {
        let model = ConfirmTransferSceneViewModel.mock(rows: { _ in [.row(row: .network(title: .network, chain: Chain.ethereum.rawValue, name: "Ethereum (ERC20)"))] })
        let networkItem = model.itemModel(for: .row(0))

        if case let .row(.network(_, _, name)) = networkItem {
            #expect(name == "Ethereum (ERC20)")
        } else {
            Issue.record("Expected network row")
        }
    }

    @Test
    func networkFeeItemModel() async throws {
        let fee = GemConfirmFee.mock(
            value: 1,
            formatted: feeAmount(asset: Primitives.Asset.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18).toGem(), value: 1, price: nil, currency: Primitives.Currency.usd.toGem()),
            amount: .amount(amount: .mock(value: 1, networkFee: 1)),
        )
        let confirmation = GemConfirmationMock(state: .mock(fee: fee), feeRates: .mock(
            rows: [.mock(kind: .priority(priority: .normal), isSelected: true)],
            showsOptions: false,
            unitType: .gwei,
            unitDecimals: 9,
            selectedTotal: 20,
            normalTotal: 20,
        ))
        let model = ConfirmTransferSceneViewModel.mock(confirmation: confirmation)

        model.state = .mock(screen: .mock(phase: .failed, failure: GemConfirmFailure(stage: .load, error: .Load(msg: "test"))))
        let errorFeeItem = model.itemModel(for: .networkFee)

        if case let .networkFee(listItem, selectable) = errorFeeItem {
            #expect(listItem.subtitle == "-")
            #expect(listItem.subtitleExtra == nil)
            #expect(selectable == false)
        } else {
            Issue.record("Expected network fee item model for error state")
        }

        _ = try await confirmation.state()
        model.state = .mock(load: .mock(fee: fee), screen: .mock(phase: .ready, hasFee: true))
        let loadedFeeItem = model.itemModel(for: .networkFee)

        if case let .networkFee(listItem, selectable) = loadedFeeItem {
            #expect(listItem.subtitle != nil)
            #expect(selectable)
        } else {
            Issue.record("Expected network fee item model with loaded fee")
        }
    }

    @Test
    func networkFeeStaysSelectableWhileReloading() {
        let model = ConfirmTransferSceneViewModel.mock(confirmation: GemConfirmationMock(feeRates: .mock(
            rows: [.mock(kind: .priority(priority: .normal), isSelected: true), .mock(kind: .priority(priority: .fast), isSelected: false)],
            showsOptions: true,
            unitType: .gwei,
            unitDecimals: 9,
            selectedTotal: 20,
            normalTotal: 20,
        )))

        model.state = .mock(load: .mock())
        let reloadingFeeItem = model.itemModel(for: .networkFee)

        if case let .networkFee(listItem, selectable) = reloadingFeeItem {
            #expect(listItem.subtitle == nil)
            #expect(listItem.hasSubtitlePlaceholder)
            #expect(selectable)
        } else {
            Issue.record("Expected network fee item model while reloading")
        }
    }

    @Test
    func aFeeChangeAndARefreshEachLeaveOneConsistentViewState() async {
        let confirmation = GemConfirmationMock(feeRates: .mock(
            rows: [.mock(kind: .priority(priority: .normal), isSelected: true), .mock(kind: .priority(priority: .fast), isSelected: false)],
            showsOptions: true,
            unitType: .gwei,
            unitDecimals: 9,
            selectedTotal: 20,
            normalTotal: 20,
        ))
        let model = ConfirmTransferSceneViewModel.mock(confirmation: confirmation)
        let expected = { confirmation.viewState(screen: model.state.screen) }

        model.changeFeeSelection(.priority(priority: .fast))
        #expect(model.viewState == expected())

        await model.load()
        #expect(model.viewState == expected())
    }

    @Test
    func fetchAfterFeeChangeReplacesTheSceneWithTheServiceAnswer() async {
        let kinds: [GemFeeRateKind] = [.priority(priority: .normal), .priority(priority: .fast)]
        let warning: GemListRow = .notice(title: .warning, message: .externallyOwnedSpenderWarning, kind: .warning)
        let model = ConfirmTransferSceneViewModel.mock(confirmation: GemConfirmationMock(
            feeRates: .mock(rows: [
                .mock(kind: .priority(priority: .normal), isSelected: true),
                .mock(kind: .priority(priority: .fast), isSelected: false),
            ], showsOptions: true, unitType: .gwei, unitDecimals: 9, selectedTotal: 20, normalTotal: 20),
            warnings: [warning],
        ))

        #expect(model.simulationWarnings == [warning], "the request's warnings show before the load")

        await model.load()
        #expect(model.viewState.feeRates?.rows.map(\.kind) == kinds)

        model.changeFeeSelection(.priority(priority: .fast))
        await model.load()

        #expect(model.simulationWarnings.isEmpty)
        #expect(model.viewState.feeRates?.rows.map(\.kind) == kinds)
    }

    @Test
    func reloadKeepsTheLoadedFeeRowUntilTheConfirmationAnswers() async {
        let confirmationMock = GemConfirmationMock(
            state: .mock(fee: nil),
            load: .success(.mock(fee: .mock())),
            feeRates: .mock(
                rows: [.mock(kind: .priority(priority: .normal), isSelected: true), .mock(kind: .priority(priority: .fast), isSelected: false)],
                showsOptions: true,
                unitType: .gwei,
                unitDecimals: 9,
                selectedTotal: 20,
                normalTotal: 20,
            ),
        )
        let model = ConfirmTransferSceneViewModel.mock(confirmation: confirmationMock)
        await model.load()

        await confirmation { reloading in
            confirmationMock.onLoad = {
                let feeItem = model.itemModel(for: .networkFee)
                guard case let .networkFee(listItem, selectable) = feeItem else { return }
                #expect(model.state.screen.phase == .loading)
                #expect(model.state.fee != nil)
                #expect(listItem.hasSubtitlePlaceholder)
                #expect(selectable)
                reloading()
            }
            model.changeFeeSelection(.priority(priority: .fast))
            await model.load()
        }
        #expect(model.viewState.feeRates?.rows.count == 2)
    }

    @Test
    func firstLoadShowsTheScreenBeforeThePreloadArrives() async {
        let confirmationMock = GemConfirmationMock(
            state: .mock(addressName: Primitives.AddressName.mock(name: "vitalik.eth").toGem(), fee: nil),
            load: .success(.mock(addressName: Primitives.AddressName.mock(name: "vitalik.eth").toGem(), fee: .mock())),
        )
        let model = ConfirmTransferSceneViewModel.mock(confirmation: confirmationMock)

        await confirmation { preloading in
            confirmationMock.onLoad = {
                #expect(model.state.screen.phase == .loading)
                #expect(model.state.load?.addressName?.name == "vitalik.eth")
                preloading()
            }
            await model.load()
        }
        #expect(model.state.fee != nil)
        #expect(model.state.load?.addressName?.name == "vitalik.eth")
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
                inputType: .transfer(asset: Primitives.Asset.mock(id: .mock(chain: .solana)).toGem()),
                recipient: .mock(memo: "Test memo"),
            ),
        )
        let memoItem = modelWithMemo.itemModel(for: .row(3))

        if case let .row(.memo(_, value, menu)) = memoItem, case let .copy(copy) = menu.first {
            #expect(value == "Test memo")
            #expect(copy.value == "Test memo")
        } else {
            Issue.record("Expected memo row")
        }

        let modelNoMemo = ConfirmTransferSceneViewModel.mock(
            data: .mock(inputType: .transfer(asset: Primitives.Asset.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18).toGem())),
        )
        #expect(modelNoMemo.sections[1].values == [.row(0), .row(1), .row(2), .details])
    }

    @Test
    func swapDetailsItemModel() {
        let swapModel = ConfirmTransferSceneViewModel.mock(
            data: .mock(inputType: .swap(
                fromAsset: Primitives.Asset.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18).toGem(),
                toAsset: Primitives.Asset.mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20).toGem(),
                swapData: .mock(),
            )),
        )
        let swapItem = swapModel.itemModel(for: .details)

        if case .swapDetails = swapItem {
            // Expected swap details
        } else {
            Issue.record("Expected swap details item model")
        }

        let transferModel = ConfirmTransferSceneViewModel.mock(
            data: .mock(inputType: .transfer(asset: Primitives.Asset.mock().toGem())),
        )
        let transferSwapItem = transferModel.itemModel(for: .details)

        if case .empty = transferSwapItem {
            // Expected empty for non-swap
        } else {
            Issue.record("Expected empty for non-swap transaction")
        }
    }

    @Test
    func errorItemModel() {
        let model = ConfirmTransferSceneViewModel.mock()
        model.state = .mock(screen: .mock(phase: .failed, failure: GemConfirmFailure(stage: .load, error: .Load(msg: "Test error"))))

        let errorItem = model.itemModel(for: .error)

        if case let .error(title, _, _) = errorItem {
            #expect(title == Localized.Errors.errorOccurred)
        } else if case .empty = errorItem {
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

            let errorItem = model.itemModel(for: .error)
            guard case let .error(_, displayError, _) = errorItem else {
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

        #expect(sections.map(\.id) == ["header", "details", "fee"], "no error section until a load fails")

        #expect(sections[0].values == [.header])
        #expect(sections[1].values == [.row(0), .row(1), .row(2), .details], "a send on a chain without memos has no app or memo row")
        #expect(sections[2].values == [.networkFee])
    }

    @Test
    func walletConnectSectionsStructure() async {
        let payload = [
            SimulationPayloadField.mock(kind: .contract, value: "0x1111111111111111111111111111111111111111", fieldType: .address, display: .primary),
            SimulationPayloadField.mock(kind: .method, value: "Approve", fieldType: .text, display: .primary),
        ]
        let rows = [
            GemSimulationPayloadRow(
                title: .contract,
                value: .address(display: "0x1111...1111", copy: addressCopy(chain: Chain.ethereum.rawValue, address: "0x1111111111111111111111111111111111111111"), explorer: BlockExplorerLink.mock().toGem()),
            ),
            GemSimulationPayloadRow(title: .method, value: .text(text: "Approve")),
        ]
        let model = ConfirmTransferSceneViewModel.mock(
            data: .mock(inputType: .generic(
                asset: Primitives.Asset.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18).toGem(),
                metadata: Primitives.ApplicationMetadata.mock().toGem(),
                extra: .mock(to: "0x1111111111111111111111111111111111111111"),
            )),
            simulation: .mock(
                warnings: [.mock(severity: .warning, warning: .tokenApproval(.mock(assetId: AssetId(chain: .ethereum, tokenId: "0x1111111111111111111111111111111111111111").identifier)))],
                payload: payload,
            ),
            load: .success(.mock(
                simulation: .mock(
                    warnings: [.notice(title: .unlimitedApproval, message: .unlimitedApprovalWarning, kind: .warning)],
                    simulation: .mock(primaryFields: rows),
                ),
            )),
            rows: { _ in
                [
                    .row(row: .app(title: .app, name: "Gem Wallet", iconUrl: nil, menu: [])),
                    .row(row: .wallet(
                        title: .wallet,
                        wallet: walletRow(wallet: Wallet.mock().toGem()),
                        menu: [.copy(copy: addressCopy(chain: Chain.ethereum.rawValue, address: "0x1"))],
                    )),
                    .row(row: .network(title: .network, chain: Chain.ethereum.rawValue, name: "Ethereum")),
                ]
            },
        )
        await model.load()
        let sections = model.sections

        #expect(sections.map(\.id) == ["header", "details", "warnings", "payload", "fee"])

        #expect(sections[1].values == [.row(0), .row(1), .row(2)])
        #expect(sections[2].values == [.warnings])
        #expect(sections[3].values == [.payload])
    }

    @Test
    func buttonDisabledWithCriticalWarnings() async {
        let model = ConfirmTransferSceneViewModel.mock(
            simulation: .mock(warnings: [.mock(severity: .critical, warning: .suspiciousSpender)]),
            load: .success(.mock(
                simulation: .mock(simulation: .mock(hasCriticalWarning: true)),
            )),
        )
        await model.load()

        #expect(model.viewState.button.state == .disabled)
    }

    @Test
    func buttonEnabledWithNoWarnings() {
        #expect(ConfirmTransferSceneViewModel.mock().viewState.button.state == .loading)
    }

    @Test
    func titleFollowsTheTransferType() {
        let send = TransactionInputType.generic(asset: Primitives.Asset.mock().toGem(), metadata: Primitives.ApplicationMetadata.mock().toGem(), extra: .mock(outputAction: .send))
        let sign = TransactionInputType.generic(asset: Primitives.Asset.mock().toGem(), metadata: Primitives.ApplicationMetadata.mock().toGem(), extra: .mock(outputAction: .sign))

        #expect(ConfirmTransferSceneViewModel.mock(data: .mock(inputType: .deposit(asset: Primitives.Asset.mock().toGem()))).title == "Deposit")
        #expect(ConfirmTransferSceneViewModel.mock(data: .mock(inputType: send)).title == Localized.Transfer.reviewRequest)
        #expect(ConfirmTransferSceneViewModel.mock(data: .mock(inputType: sign)).title == Localized.Transfer.reviewRequest)
    }

    @Test
    func scanTransactionMaliciousError() {
        let model = ConfirmTransferSceneViewModel.mock()
        model.onSelectListError(error: .ScanMalicious)

        guard case let .info(sheet) = model.isPresentingSheet else {
            Issue.record("Expected maliciousTransaction sheet")
            return
        }
        #expect(sheet.title == .maliciousTransaction)
    }

    @Test
    func scanTransactionMemoRequiredError() {
        let model = ConfirmTransferSceneViewModel.mock()
        model.onSelectListError(error: .ScanMemoRequired(symbol: "BTC"))

        guard case let .info(sheet) = model.isPresentingSheet else {
            Issue.record("Expected memoRequired sheet")
            return
        }
        #expect(sheet.description == .memoRequired(symbol: "BTC"))
    }

    @Test
    func aRefreshThatFindsTheSameProblemLeavesTheDismissedSheetClosed() async {
        let required = BigInt(21_000_000_000_000)
        let problem = GemConfirmError.InsufficientNetworkFee(
            asset: Asset.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18).toGem(),
            requirement: GemBalanceRequirement(required: required, available: 0, shortfall: required),
        )
        let model = ConfirmTransferSceneViewModel.mock(load: .success(.mock(fee: .mock(amount: .error(error: problem)))))

        await model.load()
        guard case .info = model.isPresentingSheet else {
            Issue.record("Expected the insufficient network fee sheet")
            return
        }

        model.isPresentingSheet = nil
        await model.load()
        #expect(model.isPresentingSheet == nil)
    }

    @Test
    func insufficientNetworkFeeErrorShowsRequiredAmount() {
        let model = ConfirmTransferSceneViewModel.mock()
        let required = BigInt(21_000_000_000_000)
        model.onSelectListError(error: .InsufficientNetworkFee(
            asset: Asset.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18).toGem(),
            requirement: GemBalanceRequirement(required: required, available: 0, shortfall: required),
        ))

        guard case let .info(sheet) = model.isPresentingSheet, case let .insufficientNetworkFeeBalance(required, _, _, shortfall) = sheet.description else {
            Issue.record("Expected insufficientNetworkFee sheet")
            return
        }
        #expect(required?.amount.value == 0.000021)
        #expect(shortfall?.value == 0.000021)
    }

    @Test
    func insufficientNetworkFeeBuyActionUsesSmallDefaultAmount() {
        let model = ConfirmTransferSceneViewModel.mock()
        model.onSelectListError(error: .InsufficientNetworkFee(asset: Asset.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18).toGem(), requirement: nil))

        guard case let .info(sheet) = model.isPresentingSheet, let action = sheet.action else {
            Issue.record("Expected insufficientNetworkFee sheet")
            return
        }

        model.onInfoAction(action)

        guard case let .fiatConnect(_, _, amount) = model.isPresentingSheet else {
            Issue.record("Expected fiatConnect sheet")
            return
        }
        #expect(amount != nil, "a fee sheet buys the fee amount Core names")
    }

    @Test
    func tronInsufficientBalanceActionShowsGetOptions() {
        let model = ConfirmTransferSceneViewModel.mock(data: .mock(inputType: .transfer(asset: Primitives.Asset.mock(
            id: .mock(chain: .tron, tokenId: "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"),
            name: "Tether USD",
            symbol: "USDT",
            decimals: 6,
            type: .trc20,
        ).toGem())))
        model.onSelectListError(error: .InsufficientBalance(
            asset: Asset.mock(id: .mock(chain: .tron), name: "TRON", symbol: "TRX", decimals: 6).toGem(),
            requirement: GemBalanceRequirement(required: 36_798_300, available: 36_070_000, shortfall: 728_300),
        ))

        guard case let .info(sheet) = model.isPresentingSheet, let action = sheet.action else {
            Issue.record("Expected balanceRequired sheet")
            return
        }
        #expect(sheet.title == .balanceRequired(symbol: "TRX"))
        #expect(sheet.description.text == Localized.Info.balanceRequiredDescription(
            "36.79 TRX".boldMarkdown(),
            "36.07 TRX".boldMarkdown(),
            "0.7283 TRX".boldMarkdown(),
        ))

        model.onInfoAction(action)

        guard case let .getAsset(asset, acquire) = model.isPresentingSheet else {
            Issue.record("Expected getAsset sheet")
            return
        }
        #expect(asset.id == Asset.mock(id: .mock(chain: .tron), name: "TRON", symbol: "TRX", decimals: 6).id)
        #expect(acquire.buyAmount == nil)
    }

    @Test
    func tronTokenInsufficientBalancePreservesAsset() {
        let asset = Asset.mock(id: .mock(chain: .tron, tokenId: "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"), name: "Tether USD", symbol: "USDT", decimals: 6, type: .trc20)
        let model = ConfirmTransferSceneViewModel.mock(data: .mock(inputType: .transfer(asset: asset.toGem())))
        model.onSelectListError(error: .InsufficientBalance(asset: asset.toGem(), requirement: GemBalanceRequirement(required: 2, available: 1, shortfall: 1)))

        guard case let .info(sheet) = model.isPresentingSheet, let action = sheet.action else {
            Issue.record("Expected balanceRequired sheet")
            return
        }
        model.onInfoAction(action)

        guard case let .getAsset(sheetAsset, acquire) = model.isPresentingSheet else {
            Issue.record("Expected getAsset sheet")
            return
        }
        #expect(sheetAsset.id == asset.id)
        #expect(model.assetAddress(sheetAsset).asset.id == asset.id)
        #expect(acquire.buyAmount == nil)
    }

    @Test
    func insufficientBalanceBuyActionUsesErrorAsset() {
        let asset = Asset.mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20)
        let model = ConfirmTransferSceneViewModel.mock(data: .mock(inputType: .transfer(asset: asset.toGem())))
        model.onSelectListError(error: .InsufficientBalance(asset: asset.toGem(), requirement: GemBalanceRequirement(required: 2, available: 1, shortfall: 1)))

        guard case let .info(sheet) = model.isPresentingSheet, let action = sheet.action else {
            Issue.record("Expected balanceRequired sheet")
            return
        }
        model.onInfoAction(action)

        guard case let .fiatConnect(assetAddress, _, amount) = model.isPresentingSheet else {
            Issue.record("Expected fiatConnect sheet")
            return
        }
        #expect(assetAddress.asset.id == asset.id)
        #expect(amount == nil)
    }

    @Test
    func insufficientNetworkFeeSheetShowsRequiredFeeWithFiat() {
        let asset = Asset.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18)
        let feeAsset = asset.chain.asset
        let error = GemConfirmError.InsufficientNetworkFee(
            asset: asset.toGem(),
            requirement: GemBalanceRequirement(required: 2_000_000_000_000_000, available: 0, shortfall: 2_000_000_000_000_000),
        )
        let sheet = { (prices: [AssetPrice]) in
            confirmErrorInfo(error: error, prices: prices, currency: Currency.usd.toGem(), inputAssetId: asset.id.identifier, feeAssetId: feeAsset.id.identifier).map(\.infoSheet.description.text)
        }

        #expect(sheet([AssetPrice(assetId: asset.id.identifier, price: 2000, priceChangePercentage24h: 0, updatedAt: .now)]) == Localized.Info.InsufficientNetworkFeeBalance.description(
            "0.002 ETH (~$4.00)".boldMarkdown(),
            feeAsset.name.boldMarkdown(),
            "0 ETH".boldMarkdown(),
            "0.002 ETH".boldMarkdown(),
        ))
        #expect(sheet([]) == Localized.Info.InsufficientNetworkFeeBalance.description(
            "0.002 ETH".boldMarkdown(),
            feeAsset.name.boldMarkdown(),
            "0 ETH".boldMarkdown(),
            "0.002 ETH".boldMarkdown(),
        ))
    }

    @Test
    func tronInsufficientNetworkFeeUsesFeeAsset() {
        let model = ConfirmTransferSceneViewModel.mock(data: .mock(inputType: .transfer(asset: Primitives.Asset.mock(
            id: .mock(chain: .tron, tokenId: "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"),
            name: "Tether USD",
            symbol: "USDT",
            decimals: 6,
            type: .trc20,
        ).toGem())))
        model.onSelectListError(error: .InsufficientNetworkFee(asset: Asset.mock(id: .mock(chain: .tron), name: "TRON", symbol: "TRX", decimals: 6).toGem(), requirement: nil))

        guard case let .info(sheet) = model.isPresentingSheet, let action = sheet.action, case let .acquire(sheetAsset, _) = action else {
            Issue.record("Expected insufficientNetworkFee sheet")
            return
        }
        #expect(sheetAsset.id == Asset.mock(id: .mock(chain: .tron), name: "TRON", symbol: "TRX", decimals: 6).id.identifier)
        #expect(action.title == Localized.Asset.getAsset("TRX"))

        model.onInfoAction(action)

        guard case let .getAsset(asset, acquire) = model.isPresentingSheet else {
            Issue.record("Expected getAsset sheet")
            return
        }
        #expect(asset.id == Asset.mock(id: .mock(chain: .tron), name: "TRON", symbol: "TRX", decimals: 6).id)
        #expect(acquire.buyAmount != nil)
    }

    private func verifyNonEmpty(_ model: ConfirmTransferItemModel) {
        if case .empty = model {
            Issue.record("Expected non-empty model")
        }
    }
}
