package com.potspot.app.data.local

import android.content.Context
import androidx.room.Database
import androidx.room.Room
import androidx.room.RoomDatabase

@Database(
    entities = [
        StrainEntity::class,
        TerpeneEntity::class,
        StrainTerpeneCrossRef::class,
        EffectEntity::class,
        StrainEffectCrossRef::class,
    ],
    version = 1,
    exportSchema = false,
)
abstract class PotSpotDatabase : RoomDatabase() {
    abstract fun strainDao(): StrainDao

    companion object {
        @Volatile
        private var INSTANCE: PotSpotDatabase? = null

        fun getInstance(context: Context): PotSpotDatabase {
            return INSTANCE ?: synchronized(this) {
                INSTANCE ?: Room.databaseBuilder(
                    context.applicationContext,
                    PotSpotDatabase::class.java,
                    "potspot.db"
                )
                    .fallbackToDestructiveMigration()
                    .build()
                    .also { INSTANCE = it }
            }
        }
    }
}
