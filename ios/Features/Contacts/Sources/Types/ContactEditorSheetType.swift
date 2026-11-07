// Copyright (c). Gem Wallet. All rights reserved.

enum ContactEditorSheetType: Identifiable {
    case address(ContactAddressEditorSceneViewModel.Mode)
    case avatar

    var id: String {
        switch self {
        case let .address(mode): "address-\(mode.id)"
        case .avatar: "avatar"
        }
    }
}
