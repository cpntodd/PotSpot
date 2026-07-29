package com.potspot.app

import android.app.Application
import com.potspot.app.data.local.PotSpotDatabase
import com.potspot.app.sync.SyncWorker

class PotSpotApplication : Application() {

    lateinit var database: PotSpotDatabase
        private set

    override fun onCreate() {
        super.onCreate()
        instance = this

        // Initialize Room database
        database = PotSpotDatabase.getInstance(this)

        // Schedule periodic catalog sync
        SyncWorker.schedule(this)
    }

    companion object {
        lateinit var instance: PotSpotApplication
            private set
    }
}
