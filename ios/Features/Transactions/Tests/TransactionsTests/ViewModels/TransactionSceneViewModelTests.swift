import Components
import Foundation
import struct Gemstone.GemCopy
import struct Gemstone.GemGroup
import enum Gemstone.GemListRow
import struct Gemstone.GemSwapAgain
import struct Gemstone.GemTransactionDetailItem
import enum Gemstone.GemTransactionDetailRow
import enum Gemstone.GemTransactionHeaderAction
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
@testable import Store
import Testing
@testable import Transactions
import TransactionsTestKit

@MainActor
struct TransactionSceneViewModelTests {
    @Test
    func nftHeaderAction() {
        let assetId = NFTAssetId(chain: .ethereum, contractAddress: "0xasset", tokenId: "1")
        var selectedAction: GemTransactionHeaderAction?
        let model = TransactionSceneViewModel.mock(
            type: .transferNFT,
            metadata: .object(["assetId": .string(assetId.identifier), "name": .string("NFT")]),
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
            metadata: .object(["assetId": .string(NFTAssetId.mock().identifier), "name": .string("NFT")]),
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
        #expect(ids(swapped).contains("swap_again"), "a signing wallet offers to swap again")

        let watching = TransactionSceneViewModel.mock(
            type: .swap,
            state: .confirmed,
            swapToAsset: .mock(id: AssetId(chain: .bitcoin, tokenId: nil)),
            wallet: .mock(type: .view),
        )
        #expect(ids(watching).contains("swap_again") == false, "a watch-only wallet cannot sign a swap")
    }

    @Test
    func swapAgainOpensTheSwapOfItsAssets() throws {
        var selectedAction: GemTransactionHeaderAction?
        let model = TransactionSceneViewModel.mock(
            type: .swap,
            state: .confirmed,
            asset: .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18),
            swapToAsset: .mock(id: AssetId(chain: .bitcoin, tokenId: nil)),
            onHeaderAction: { selectedAction = $0 },
        )
        let swap = try #require(rows(model).compactMap { row -> GemSwapAgain? in
            guard case let .swapAgain(_, swap) = row else { return nil }
            return swap
        }.first)

        model.onSelectSwapAgain(swap)

        #expect(swap.fromAssetId != swap.toAssetId)
        #expect(selectedAction == .swap(fromAssetId: swap.fromAssetId, toAssetId: swap.toAssetId))
    }

    @Test
    func memoItemModel() {
        let withMemo = listRows(TransactionSceneViewModel.mock(asset: .mock(id: .mock(chain: .cosmos)), memo: "Test memo"))
        #expect(withMemo.contains(.memo(title: .memo, value: "Test memo", menu: [.copy(copy: GemCopy(kind: .plain, value: "Test memo", display: "Test memo"))])))

        #expect(ids(TransactionSceneViewModel.mock(asset: .mock(id: .mock(chain: .cosmos)), memo: nil)).contains("memo") == false)
        #expect(ids(TransactionSceneViewModel.mock(asset: .mock(id: .mock(chain: .cosmos)), memo: "")).contains("memo") == false)
    }

    @Test
    func explorerLinkItemModel() {
        #expect(listRows(TransactionSceneViewModel.mock(id: .mock(hash: "1"))).contains(.explorer(title: .viewOn(name: "Blockchair"), url: "https://blockchair.com/bitcoin/transaction/1")))
    }

    @Test
    func sectionsComeFromCoreWithOnlyTheRowsTheTransactionHas() {
        let transfer = TransactionSceneViewModel.mock(asset: .mock(id: .mock(chain: .cosmos)), memo: "gm").sections
        #expect(transfer.map(itemIds) == [["header"], ["date", "status", "participant", "memo", "network"], ["fee"], ["explorer"]])

        let swap = TransactionSceneViewModel.mock(
            type: .swap,
            state: .inTransit,
            asset: .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18),
            swapToAsset: .mock(id: .mock(chain: .near), name: "NEAR", symbol: "NEAR", decimals: 24),
            confirmationEtaSeconds: 720,
        ).sections
        #expect(swap.map(itemIds) == [["header"], ["swap_progress"], ["date", "status", "rate", "network", "provider"], ["fee"], ["explorer"]])
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

    private func details(_ group: GemGroup) -> [GemTransactionDetailItem] {
        group.items.compactMap { item in
            guard case let .transactionDetail(detail) = item else { return nil }
            return detail
        }
    }

    private func itemIds(_ group: GemGroup) -> [String] {
        details(group).map(\.id)
    }

    private func details(_ model: TransactionSceneViewModel) -> [GemTransactionDetailItem] {
        model.sections.flatMap(details)
    }

    private func ids(_ model: TransactionSceneViewModel) -> [String] {
        details(model).map(\.id)
    }

    private func rows(_ model: TransactionSceneViewModel) -> [GemTransactionDetailRow] {
        details(model).map(\.row)
    }

    private func listRows(_ model: TransactionSceneViewModel) -> [GemListRow] {
        rows(model).compactMap { row in
            guard case let .row(listRow) = row else { return nil }
            return listRow
        }
    }
}

@MainActor
struct TransactionSceneViewModelLoadTests {
    @Test
    func aStoredTransactionShowsItsDetailsAtOnce() async {
        let service = GemTransactionsServiceMock()
        let model = TransactionSceneViewModel.mock(transactionsService: service)

        #expect(model.details != nil)

        await model.load()
        #expect(model.loadState == .data)
        #expect(service.loadedTransactionIds == [.mock()])
    }

    @Test
    func aTransactionTheWalletLacksShowsNothingAndAsksCoreToLoadIt() async {
        let service = GemTransactionsServiceMock()
        let model = TransactionSceneViewModel.mock(isStored: false, transactionsService: service)

        #expect(model.details == nil)
        #expect(model.loadState == .loading)
        #expect(model.title.isEmpty)
        #expect(model.explorerLink == nil)

        await model.load()
        #expect(model.loadState == .data)
        #expect(service.loadedTransactionIds == [.mock()])
    }

    @Test
    func aLoadedTransactionIsFollowedByItsRecord() {
        let model = TransactionSceneViewModel.mock(isStored: false)
        #expect(model.query.request.base == TransactionQuery(walletId: Wallet.mock().id, key: .transaction(.mock())))

        model.onRecordChange(7)

        #expect(model.query.request.base == TransactionQuery(walletId: Wallet.mock().id, key: .record(7)))
    }

    @Test
    func aFailedLoadKeepsItsErrorForTryAgain() async {
        let service = GemTransactionsServiceMock()
        service.loadState = .error(error: .Api(msg: "not found"))
        let model = TransactionSceneViewModel.mock(isStored: false, transactionsService: service)

        await model.load()
        #expect(model.loadState == .error(error: .Api(msg: "not found")))
    }
}
