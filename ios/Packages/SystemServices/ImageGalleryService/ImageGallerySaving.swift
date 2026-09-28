// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public protocol ImageGallerySaving: Sendable {
    func saveImageFromURL(_ url: URL) async throws(ImageGalleryServiceError)
}

extension ImageGalleryService: ImageGallerySaving {}
