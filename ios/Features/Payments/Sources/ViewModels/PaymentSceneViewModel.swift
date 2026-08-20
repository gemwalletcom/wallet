// Copyright (c). Gem Wallet. All rights reserved.

import BalanceService
import BigInt
import Components
import Formatters
import Foundation
import GemstonePrimitives
import Localization
import PaymentService
import Primitives
import PrimitivesComponents

@Observable
@MainActor
public final class PaymentSceneViewModel {
    private static let amountFormatter = ValueFormatter(style: .short)
    private static let priceFormatter = ValueFormatter(style: .full)

    private let wallet: Wallet
    private let link: PaymentLink
    private let paymentService: any PaymentServiceable
    private let balanceService: BalanceService
    private let onTransferAction: TransferDataAction
    private let onComplete: VoidAction

    public var state: PaymentState
    public var isPresentingSheet: PaymentSheetType?

    public init(
        wallet: Wallet,
        link: PaymentLink,
        quotes: PaymentQuotes,
        paymentService: any PaymentServiceable,
        balanceService: BalanceService,
        onTransferAction: TransferDataAction,
        onComplete: VoidAction,
    ) {
        self.wallet = wallet
        self.link = link
        self.paymentService = paymentService
        self.balanceService = balanceService
        self.onTransferAction = onTransferAction
        self.onComplete = onComplete
        state = PaymentState(quotes: quotes)
    }

    var title: String { Localized.Transfer.paymentTitle }

    var recipientTitle: String { Localized.Transfer.Recipient.title }

    var walletTitle: String { Localized.Common.wallet }

    var payWithTitle: String { Localized.Transfer.payWith }

    var expiresTitle: String { Localized.Transfer.paymentExpiresIn }

    var errorTitle: String { Localized.Errors.errorOccurred }

    var walletText: String { wallet.name }

    var walletAssetImage: AssetImage {
        WalletViewModel(wallet: wallet).avatarImage
    }

    var preview: AppPreviewModel {
        AppPreviewModel(
            assetImage: selectedItem?.assetImage ?? AssetImage(imageURL: state.quotes.merchant.iconUrl?.asURL),
            name: selectedItem?.amountText ?? state.quotes.merchant.name,
            subtitleSymbol: state.quotes.price.flatMap(priceText),
        )
    }

    var selectedItem: PaymentQuoteItem? {
        state.selectedQuote.map(item)
    }

    var quoteItems: [PaymentQuoteItem] {
        state.quotes.quotes.map(item)
    }

    var showsQuoteSelection: Bool {
        quoteItems.count > 1
    }

    var quotes: PaymentQuotes {
        state.quotes
    }

    var quotesModel: PaymentQuotesViewModel {
        PaymentQuotesViewModel(
            state: .data(.plain(quoteItems)),
            selectedItems: quoteItems.filter { $0.id == state.selectedQuoteId },
            selectionType: .checkmark,
        )
    }

    var buttonModel: PaymentButtonViewModel {
        PaymentButtonViewModel(
            state: state,
            onAction: { [weak self] in self?.onSelectButton() },
        )
    }
}

// MARK: - Actions

extension PaymentSceneViewModel {
    func fetch() async {
        state.refresh = .loading
        do {
            switch try await paymentService.getOptions(link: link, addresses: addresses) {
            case let .quotes(quotes): state.replace(with: quotes)
            case .outcome: onComplete?()
            }
        } catch {
            state.refresh.setError(error)
        }
    }

    public func awaitExpiry() async {
        guard let expiresAt = state.quotes.expiresAt else { return }
        try? await Task.sleep(for: .seconds(max(0, expiresAt.timeIntervalSinceNow)))
        guard !Task.isCancelled else { return }
        state.isExpired = true
    }

    func onSelectQuotes() {
        isPresentingSheet = .quotes
    }

    func onFinishQuotesSelection(items: [PaymentQuoteItem]) {
        state.select(quoteId: items.first?.id)
        isPresentingSheet = .none
    }

    func onSelectButton() {
        switch buttonModel.buttonAction {
        case .tryAgain:
            Task { await retry() }
        case .collectData:
            presentDataCollection()
        case .confirm:
            Task { await confirm() }
        }
    }

    public func onCompleteDataCollection() {
        isPresentingSheet = .none
        state.completeDataCollection()
    }
}

// MARK: - Private

extension PaymentSceneViewModel {
    private func presentDataCollection() {
        isPresentingSheet = state.selectedQuote?.collectDataUrl
            .flatMap(URL.init(string:))
            .map(PaymentSheetType.dataCollection)
    }

    private var addresses: [ChainAddress] {
        wallet.accounts.map { ChainAddress(chain: $0.chain, address: $0.address) }
    }

    private func item(for quote: PaymentQuote) -> PaymentQuoteItem {
        let asset = quote.assetId.chain.asset
        return PaymentQuoteItem(
            quote: quote,
            asset: asset,
            balance: balanceText(assetId: quote.assetId, asset: asset),
            formatter: Self.amountFormatter,
        )
    }

    private func balanceText(assetId: AssetId, asset: Asset) -> String {
        guard let balance = try? balanceService.getBalance(walletId: wallet.id, assetId: assetId) else { return .empty }
        return Self.amountFormatter.string(balance.available, decimals: asset.decimals.asInt, currency: asset.symbol)
    }

    private func priceText(_ price: PaymentPrice) -> String? {
        guard let value = BigInt(price.value) else { return .none }
        return Self.priceFormatter.string(value, decimals: price.decimals.asInt, currency: price.symbol)
    }

    private func retry() async {
        if state.refresh.isError || state.isExpired {
            await fetch()
        } else {
            await confirm()
        }
    }

    func confirm() async {
        guard !state.transferData.isLoading else { return }
        guard let quote = state.selectedQuote else { return }
        state.transferData = .loading
        do {
            let quoteData = try await paymentService.getQuoteData(quote: quote, addresses: addresses)
            let transferData = try PaymentTransferDataFactory.payment(quoteData: quoteData, merchant: state.quotes.merchant)
            onTransferAction?(transferData)
            state.transferData = .noData
        } catch {
            state.transferData.setError(error)
        }
    }
}
