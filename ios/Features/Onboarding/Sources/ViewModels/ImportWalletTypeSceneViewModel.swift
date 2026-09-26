import Components
import Foundation
import class Gemstone.GemChainService
import struct Gemstone.GemImportWalletTypes
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct ImportWalletTypeSceneViewModel: Hashable {
    public init() {}

    var title: String {
        Localized.Wallet.Import.title
    }

    func types(for searchText: String) -> GemImportWalletTypes {
        GemChainService.shared.importWalletTypes(query: searchText)
    }

    func multicoinListItem(_ types: GemImportWalletTypes) -> ListItemModel {
        ListItemModel(title: types.multicoin.text, imageStyle: .asset(assetImage: AssetImage.image(Images.Logo.logo)))
    }
}
