import protocol Gemstone.GemBalanceServiceProtocol
import Components
import Foundation
import protocol Gemstone.GemAssetsServiceProtocol
import GemstoneServices
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import SwiftUI

@Observable
@MainActor
public final class ReceiveViewModel: Sendable {
    var qrSize: CGFloat {
        UIDevice.current.userInterfaceIdiom == .pad ? 180 : 260
    }

    private(set) var assetModel: AssetViewModel
    private(set) var address: String

    var presentation: ReceivePresentationType?
    var renderedImage: UIImage?

    private let wallet: Wallet
    private let balanceService: any GemBalanceServiceProtocol
    private let assetsService: any GemAssetsServiceProtocol
    private let generator = QRCodeGenerator()
    let networkAssetIds: [AssetId]

    private init(
        asset: Asset,
        associations: [AssetAssociation],
        wallet: Wallet,
        address: String,
        balanceService: any GemBalanceServiceProtocol,
        assetsService: any GemAssetsServiceProtocol,
    ) {
        assetModel = AssetViewModel(asset: asset)
        self.wallet = wallet
        self.address = address
        self.balanceService = balanceService
        self.assetsService = assetsService

        networkAssetIds = ([asset.id] + associations.map(\.assetId))
            .filter { assetId in wallet.accounts.contains { $0.chain == assetId.chain } }
            .unique()
    }

    public convenience init(
        assetData: AssetData,
        wallet: Wallet,
        balanceService: any GemBalanceServiceProtocol,
        assetsService: any GemAssetsServiceProtocol,
    ) {
        self.init(
            asset: assetData.asset,
            associations: assetData.associations,
            wallet: wallet,
            address: assetData.account.address,
            balanceService: balanceService,
            assetsService: assetsService,
        )
    }

    public convenience init(
        assetAddress: AssetAddress,
        wallet: Wallet,
        balanceService: any GemBalanceServiceProtocol,
        assetsService: any GemAssetsServiceProtocol,
    ) {
        self.init(
            asset: assetAddress.asset,
            associations: [],
            wallet: wallet,
            address: assetAddress.address,
            balanceService: balanceService,
            assetsService: assetsService,
        )
    }

    var title: String {
        Localized.Receive.title("")
    }

    var addressShort: String {
        AddressFormatter(style: .short, address: address, chain: assetModel.asset.chain).value()
    }

    var shareTitle: String {
        Localized.Common.share
    }

    var copyTitle: String {
        Localized.Common.copy
    }

    var warningMessage: String {
        [Localized.Receive.warning(assetModel.symbol.boldMarkdown(), assetModel.networkFullName.boldMarkdown()), memoWarningText]
            .compactMap(\.self)
            .joined(separator: " ")
    }

    private var memoWarningText: String? {
        switch assetModel.asset.chain {
        case .xrp where assetModel.asset.chain.isMemoSupported: Localized.Wallet.Receive.noDestinationTagRequired
        case _ where assetModel.asset.chain.isMemoSupported: Localized.Wallet.Receive.noMemoRequired
        default: nil
        }
    }

    var copyModel: CopyTypeViewModel {
        CopyTypeViewModel(
            type: .address(assetModel.asset, address: addressShort),
            copyValue: address,
        )
    }

    var showNetworkSelector: Bool {
        networkAssetIds.count > 1
    }

    var networkSelectorModel: ReceiveNetworkSelectorViewModel {
        ReceiveNetworkSelectorViewModel(
            assetIds: networkAssetIds,
        )
    }

    var isPresentingSheet: ReceivePresentationType? {
        get {
            switch presentation {
            case .share, .networkSelector: presentation
            case .copy, nil: nil
            }
        }
        set {
            presentation = newValue
        }
    }

    var isPresentingCopyToast: Bool {
        get {
            if case .copy = presentation {
                return true
            }
            return false
        }
        set {
            presentation = newValue ? .copy : nil
        }
    }

    func activityItems(qrImage: UIImage?) -> [Any] {
        if let qrImage {
            return [qrImage, address]
        }
        return [address]
    }

    private func enableAsset() async {
        do {
            try await balanceService.setAssetsEnabled(wallet: wallet, assetIds: [assetModel.asset.id], enabled: true)
        } catch {
            debugLog("ReceiveViewModel enableAsset error: \(error)")
        }
    }

    private func prefetchAssociations() async {
        do {
            try await assetsService.syncMissingAssets(
                for: networkAssetIds.filter { $0 != assetModel.asset.id },
            )
        } catch {
            debugLog("ReceiveViewModel prefetchAssociations error: \(error)")
        }
    }

    private func generateQRCode() async -> UIImage? {
        await generator.generate(
            from: address,
            size: CGSize(
                width: qrSize,
                height: qrSize,
            ),
            logo: UIImage.name("logo-dark"),
        )
    }
}

// MARK: - Actions

extension ReceiveViewModel {
    func onTaskOnce() {
        Task {
            await enableAsset()
            await prefetchAssociations()
        }
    }

    func onSelectNetwork() {
        presentation = .networkSelector
    }

    func onFinishNetworkSelection(_ items: [ReceiveNetworkItem]) {
        presentation = nil
        guard let assetId = items.first?.assetId, assetId != assetModel.asset.id else { return }

        Task {
            do {
                let asset = try await assetsService.ensureAsset(for: assetId)
                let account = try wallet.account(for: asset.chain)
                assetModel = AssetViewModel(asset: asset)
                address = account.address
                renderedImage = await generateQRCode()
                await enableAsset()
            } catch {
                debugLog("ReceiveViewModel onFinishNetworkSelection error: \(error)")
            }
        }
    }

    func onShareSheet() {
        presentation = .share
    }

    func onCopyAddress() {
        presentation = .copy
    }

    func onLoadImage() async {
        renderedImage = await generateQRCode()
    }
}
