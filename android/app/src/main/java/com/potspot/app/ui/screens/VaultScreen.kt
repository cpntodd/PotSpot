package com.potspot.app.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.potspot.app.PotSpotApplication
import com.potspot.app.data.local.StrainEntity
import com.potspot.app.data.repository.StrainRepository

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun VaultScreen(onBack: () -> Unit) {
    val db = PotSpotApplication.instance.let { app ->
        com.potspot.app.data.local.PotSpotDatabase.getInstance(app)
    }
    val repository = remember { StrainRepository(db) }
    val savedStrains by repository.getSavedStrains().collectAsState(initial = emptyList())

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("My Vault") },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, "Back")
                    }
                }
            )
        }
    ) { padding ->
        if (savedStrains.isEmpty()) {
            Box(modifier = Modifier.padding(padding).fillMaxSize()) {
                Text(
                    "No saved strains yet. Browse the catalog and save strains to view them offline.",
                    modifier = Modifier.padding(32.dp),
                )
            }
        } else {
            LazyColumn(
                modifier = Modifier.padding(padding),
                contentPadding = PaddingValues(16.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                items(savedStrains) { strain ->
                    VaultStrainItem(strain = strain)
                }
            }
        }
    }
}

@Composable
fun VaultStrainItem(strain: StrainEntity) {
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(strain.name, style = MaterialTheme.typography.titleMedium)
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(strain.strainType.uppercase())
                strain.thcPercentage?.let { Text("THC $it%") }
            }
        }
    }
}
