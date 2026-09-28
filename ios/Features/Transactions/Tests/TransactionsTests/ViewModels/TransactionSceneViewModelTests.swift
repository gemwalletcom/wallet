import Components
import Foundation
import struct Gemstone.GemCopy
import enum Gemstone.GemListRow
import struct Gemstone.GemSwapAgain
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
        #expect(rows(swapped).map(kind).contains("swapAgain"), "a signing wallet offers to swap again")

        let watching = TransactionSceneViewModel.mock(
            type: .swap,
            state: .confirmed,
            swapToAsset: .mock(id: AssetId(chain: .bitcoin, tokenId: nil)),
            wallet: .mock(type: .view),
        )
        #expect(rows(watching).map(kind).contains("swapAgain") == false, "a watch-only wallet cannot sign a swap")
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

        #expect(listRows(TransactionSceneViewModel.mock(asset: .mock(id: .mock(chain: .cosmos)), memo: nil)).contains { kind($0) == "memo" } == false)
        #expect(listRows(TransactionSceneViewModel.mock(asset: .mock(id: .mock(chain: .cosmos)), memo: "")).contains { kind($0) == "memo" } == false)
    }

    @Test
    func explorerLinkItemModel() {
        #expect(listRows(TransactionSceneViewModel.mock(id: .mock(hash: "1"))).contains(.explorer(title: .viewOn(name: "Blockchair"), url: "https://blockchair.com/bitcoin/transaction/1")))
    }

    @Test
    func sectionsComeFromCoreWithOnlyTheRowsTheTransactionHas() {
        let transfer = TransactionSceneViewModel.mock(asset: .mock(id: .mock(chain: .cosmos)), memo: "gm").sections
        #expect(transfer.map { $0.values.map(kind) } == [["header"], ["date", "status", "participant", "memo", "network"], ["fee"], ["explorer"]])

        let swap = TransactionSceneViewModel.mock(
            type: .swap,
            state: .inTransit,
            asset: .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18),
            swapToAsset: .mock(id: .mock(chain: .near), name: "NEAR", symbol: "NEAR", decimals: 24),
            confirmationEtaSeconds: 720,
        ).sections
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

    private func rows(_ model: TransactionSceneViewModel) -> [GemTransactionDetailRow] {
        model.sections.flatMap(\.values)
    }

    private func listRows(_ model: TransactionSceneViewModel) -> [GemListRow] {
        rows(model).compactMap { row in
            guard case let .row(listRow) = row else { return nil }
            return listRow
        }
    }

    private func kind(_ row: GemTransactionDetailRow) -> String {
        switch row {
        case let .row(listRow): kind(listRow)
        case .participant: "participant"
        case .header: "header"
        case .swapProgress: "swapProgress"
        case .swapAgain: "swapAgain"
        case .fee: "fee"
        }
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
}
