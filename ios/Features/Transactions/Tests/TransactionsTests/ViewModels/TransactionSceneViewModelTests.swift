import Components
import Foundation
import enum Gemstone.GemListRow
import enum Gemstone.GemTransactionDetailRow
import enum Gemstone.GemTransactionHeaderAction
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Localization
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
@testable import Store
import Style
import Testing
@testable import Transactions
import TransactionsTestKit

@MainActor
struct TransactionSceneViewModelTests {
    @Test
    func itemModelReturnsNonEmpty() {
        let model = TransactionSceneViewModel.mock()

        for row in model.sections.flatMap(\.values) {
            verifyNonEmpty(model.item(for: row))
        }
    }

    @Test
    func headerItemModel() {
        let model = TransactionSceneViewModel.mock(
            type: TransactionType.transfer,
            direction: TransactionDirection.outgoing,
        )
        let itemModel = model.item(for: GemTransactionDetailRow.header)

        verifyNonEmpty(itemModel)
    }

    @Test
    func nftHeaderAction() {
        let assetId = NFTAssetId(chain: .ethereum, contractAddress: "0xasset", tokenId: "1")
        var selectedAction: GemTransactionHeaderAction?
        let model = TransactionSceneViewModel.mock(
            type: .transferNFT,
            metadata: .encode(TransactionNFTTransferMetadata(assetId: assetId, name: "NFT")),
            onHeaderAction: { selectedAction = $0 },
        )

        #expect(model.onTransactionHeaderTap != nil)

        model.onTransactionHeaderTap?(.header)

        #expect(selectedAction == .nft(assetId: assetId.identifier))
    }

    @Test
    func nftHeaderActionRequiresHandler() {
        let model = TransactionSceneViewModel.mock(
            type: .transferNFT,
            metadata: .encode(TransactionNFTTransferMetadata(assetId: .mock(), name: "NFT")),
        )

        #expect(model.onTransactionHeaderTap == nil)
    }

    @Test
    func swapAgainIsHiddenForAWatchOnlyWallet() {
        let swapped = TransactionSceneViewModel.mock(
            type: .swap,
            state: .confirmed,
            swapToAsset: .mock(id: AssetId(chain: .bitcoin, tokenId: nil)),
        )
        guard case .swapAgain = swapped.item(for: GemTransactionDetailRow.swapAgain) else {
            Issue.record("a signing wallet offers to swap again")
            return
        }

        let watching = TransactionSceneViewModel.mock(
            type: .swap,
            state: .confirmed,
            swapToAsset: .mock(id: AssetId(chain: .bitcoin, tokenId: nil)),
            wallet: .mock(type: .view),
        )
        guard case .empty = watching.item(for: GemTransactionDetailRow.swapAgain) else {
            Issue.record("a watch-only wallet cannot sign a swap")
            return
        }
    }

    @Test
    func swapButtonItemModel() {
        let swapModel = TransactionSceneViewModel.mock(type: TransactionType.swap, state: TransactionState.confirmed)
        let swapItem = swapModel.item(for: GemTransactionDetailRow.swapAgain)

        if case .empty = swapItem {
        } else if case .swapAgain = swapItem {
        } else {
            Issue.record("Unexpected swap button model type")
        }

        let transferModel = TransactionSceneViewModel.mock(type: TransactionType.transfer)
        let transferItem = transferModel.item(for: GemTransactionDetailRow.swapAgain)

        if case .empty = transferItem {
        } else {
            Issue.record("Expected empty for non-swap transaction")
        }
    }

    @Test
    func dateItemModel() {
        #expect(listRows(TransactionSceneViewModel.mock()).contains { kind($0) == "date" })
    }

    @Test
    func statusItemModel() {
        #expect(statusRow(TransactionSceneViewModel.mock(state: .confirmed)) == .label(
            title: .status,
            text: .transactionState(state: .confirmed),
            tone: .positive,
            info: .transactionStatus(state: .confirmed, tone: .success),
            progress: false,
        ))
        #expect(statusRow(TransactionSceneViewModel.mock(state: .pending)) == .label(
            title: .status,
            text: .transactionState(state: .pending),
            tone: .warning,
            info: .transactionStatus(state: .pending, tone: .pending),
            progress: true,
        ))
        #expect(statusRow(TransactionSceneViewModel.mock(state: .inTransit)) == .label(
            title: .status,
            text: .transactionState(state: .inTransit),
            tone: .warning,
            info: .transactionStatus(state: .inTransit, tone: .pending),
            progress: true,
        ))
    }

    @Test
    func swapProgressItemModel() {
        let model = TransactionSceneViewModel.mock(type: .swap, state: .inTransit, asset: .mockEthereum(), swapToAsset: .mockNear(), confirmationEtaSeconds: 720)

        if case let .swapProgress(progress) = model.item(for: GemTransactionDetailRow.swapProgress) {
            #expect(progress.transfer.title == Localized.Transfer.title)
            #expect(progress.transfer.subtitle == "1 ETH (Ethereum)")
            #expect(progress.transfer.state.step == .completed)
            #expect(progress.swap.title == Localized.Wallet.swap)
            #expect(progress.swap.subtitle == "NEAR Intents")
            #expect(progress.swap.state.step == .pending)
            #expect(progress.estimatedTime == "≈ 12 min")
        } else {
            Issue.record("Expected swap progress for in-transit cross-chain swap")
        }
        #expect(estimatedConfirmation(model) == nil)
        if case .empty = TransactionSceneViewModel.mock(type: .transfer, state: .pending).item(for: GemTransactionDetailRow.swapProgress) {} else {
            Issue.record("Expected no swap progress for a transfer")
        }
    }

    @Test
    func participantItemModel() {
        let modelWithAddresses = TransactionSceneViewModel.mock(
            type: .transfer,
            direction: .incoming,
            from: "0xSenderAddress",
            to: "0xRecipientAddress",
        )

        if case let .participant(item) = modelWithAddresses.item(for: GemTransactionDetailRow.participant) {
            #expect(item.title == Localized.Transaction.sender)
            #expect(item.account.address == "0xSenderAddress")
        } else {
            Issue.record("Expected participant item for incoming transfer")
        }

        let swapModel = TransactionSceneViewModel.mock(type: TransactionType.swap)
        if case .empty = swapModel.item(for: GemTransactionDetailRow.participant) {
        } else {
            Issue.record("Expected empty for swap participant")
        }
    }

    @Test
    func memoItemModel() {
        let withMemo = listRows(TransactionSceneViewModel.mock(asset: .mock(id: .mock(.cosmos)), memo: "Test memo"))
        #expect(withMemo.contains(.memo(value: "Test memo", copy: "Test memo")))

        #expect(listRows(TransactionSceneViewModel.mock(asset: .mock(id: .mock(.cosmos)), memo: nil)).contains { kind($0) == "memo" } == false)
        #expect(listRows(TransactionSceneViewModel.mock(asset: .mock(id: .mock(.cosmos)), memo: "")).contains { kind($0) == "memo" } == false)
    }

    @Test
    func networkItemModel() {
        #expect(listRows(TransactionSceneViewModel.mock()).contains(.network(title: .network, chain: Chain.bitcoin.rawValue, name: "Bitcoin")))
    }

    @Test
    func providerItemModel() {
        #expect(listRows(TransactionSceneViewModel.mock()).contains { kind($0) == "provider" } == false)
    }

    @Test
    func feeItemModel() {
        let model = TransactionSceneViewModel.mock()

        if case let .fee(item) = model.item(for: GemTransactionDetailRow.fee) {
            #expect(item.title == Localized.Transfer.networkFee)
            #expect(item.infoAction != nil)
        } else {
            Issue.record("Expected listItem for fee")
        }
    }

    @Test
    func explorerLinkItemModel() {
        #expect(listRows(TransactionSceneViewModel.mock()).contains(.explorer(name: "Blockchair", url: "https://blockchair.com/bitcoin/transaction/1")))
    }

    @Test
    func sectionsComeFromCoreWithOnlyTheRowsTheTransactionHas() {
        let transfer = TransactionSceneViewModel.mock(asset: .mock(id: .mock(.cosmos)), memo: "gm").sections
        #expect(transfer.map { $0.values.map(kind) } == [["header"], ["date", "status", "participant", "memo", "network"], ["fee"], ["explorer"]])

        let swap = TransactionSceneViewModel.mock(type: .swap, state: .inTransit, asset: .mockEthereum(), swapToAsset: .mockNear(), confirmationEtaSeconds: 720).sections
        #expect(swap.map { $0.values.map(kind) } == [["header"], ["swapProgress"], ["date", "status", "rate", "network", "provider"], ["fee"], ["explorer"]])
    }

    @Test
    func estimatedConfirmationItemModel() throws {
        let pending = TransactionSceneViewModel.mock(state: .pending, confirmationEtaSeconds: 720)

        guard case let .duration(_, parts, info, estimate) = try #require(estimatedConfirmation(pending)) else {
            Issue.record("Expected estimated confirmation row")
            return
        }
        #expect(estimate)
        #expect(info != nil)
        #expect(EstimatedConfirmationFormatter(locale: Locale(identifier: "en_US")).string(parts: parts) == "≈ 12 min")
        #expect(estimatedConfirmation(TransactionSceneViewModel.mock(state: .confirmed, confirmationEtaSeconds: 720)) == nil)
    }

    private func estimatedConfirmation(_ model: TransactionSceneViewModel) -> GemListRow? {
        listRows(model).first {
            guard case .duration(.estimatedConfirmation, _, _, _) = $0 else { return false }
            return true
        }
    }

    private func listRows(_ model: TransactionSceneViewModel) -> [GemListRow] {
        model.sections.flatMap(\.values).compactMap { row in
            guard case let .row(listRow) = row else { return nil }
            return listRow
        }
    }

    private func statusRow(_ model: TransactionSceneViewModel) -> GemListRow? {
        listRows(model).first { kind($0) == "status" }
    }

    private func kind(_ row: GemTransactionDetailRow) -> String {
        guard case let .row(listRow) = row else { return "\(row)" }
        return kind(listRow)
    }

    private func kind(_ row: GemListRow) -> String {
        switch row {
        case let .date(title, _), let .network(title, _, _), let .text(title, _): "\(title)"
        case .memo: "memo"
        case let .label(title, _, _, _, _): "\(title)"
        case let .amount(title, _, _): "\(title)"
        case let .rate(title, _, _): "\(title)"
        case .explorer: "explorer"
        case .provider: "provider"
        default: "\(row)"
        }
    }

    private func verifyNonEmpty(_ model: TransactionItemModel) {
        if case .empty = model {
            Issue.record("Expected non-empty model")
        }
    }
}
