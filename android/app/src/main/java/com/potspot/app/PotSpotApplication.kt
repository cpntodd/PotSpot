package com.potspot.app

import android.app.Application

/**
 * PotSpot application class.
 * Initializes global dependencies (database, sync manager, etc.).
 */
class PotSpotApplication : Application() {

    override fun onCreate() {
        super.onCreate()
        instance = this

        // TODO: Initialize Room database
        // TODO: Initialize SyncManager
        // TODO: Set up certificate pinning
    }

    companion object {
        lateinit var instance: PotSpotApplication
            private set
    }
}
