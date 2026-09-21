// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PhotosUI
import SwiftUI
import UIKit

private let imageCompressionQuality = 0.9

extension PhotosPickerItem {
    func imageAttachment() async throws -> Data? {
        guard let data = try await loadTransferable(type: Data.self) else { return nil }
        guard let image = UIImage(data: data) else { return nil }
        return image.compress(compressionQuality: imageCompressionQuality)
    }
}
