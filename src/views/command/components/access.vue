<script setup lang="ts">
import { ref, reactive, computed, onMounted, onBeforeUnmount } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElNotification } from "element-plus";
import { FolderOpened, Connection } from "@element-plus/icons-vue";

const selectedFiles = ref([]);
const isLoading = ref(false);
const runtime = ref(0.0);
const tableRef = ref(null);
const windowHeight = ref(window.innerHeight);
const data = reactive({
  filePath: "",
  fileFormats: ["mdb", "accdb"],
  sep: "|"
});

const formHeight = computed(() => {
  const height = 205;
  return windowHeight.value - height;
});

const updateWindowHeight = () => {
  windowHeight.value = window.innerHeight;
};

onMounted(() => {
  window.addEventListener("resize", updateWindowHeight);
});

onBeforeUnmount(() => {
  window.removeEventListener("resize", updateWindowHeight);
});

listen("runtime", (event: any) => {
  runtime.value = event.payload;
});
listen("access_err", (event: any) => {
  const accessErr = event.payload;
  ElNotification({
    title: "Access Error",
    message: accessErr,
    position: "bottom-right",
    type: "error",
    duration: 10000
  });
  isLoading.value = false;
});

// open file
async function selectFile() {
  selectedFiles.value = [];
  const selected = await open({
    multiple: true,
    filters: [
      {
        name: "Access",
        extensions: data.fileFormats
      }
    ]
  });
  if (Array.isArray(selected)) {
    data.filePath = selected.join("|").toString();
    const nonEmptyRows = selected.filter((row: any) => row.trim() !== "");
    selectedFiles.value = nonEmptyRows.map((file: any) => {
      return { filename: file };
    });
  } else if (selected === null) {
    ElNotification({
      title: "File not found",
      message: "未选择文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  } else {
    data.filePath = selected;
  }
}

// convert data
async function accessData() {
  if (data.filePath === "") {
    ElNotification({
      title: "File not found",
      message: "未选择文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }
  if (data.filePath !== "") {
    isLoading.value = true;

    await invoke("access", {
      filePath: data.filePath,
      sep: data.sep
    });

    isLoading.value = false;
    ElNotification({
      message: "Convert done, elapsed time: " + runtime.value,
      position: "bottom-right",
      type: "success",
      duration: 5000
    });
  }
}
</script>

<template>
  <el-form class="page-container" :style="formHeight">
    <el-form>
      <div
        style="
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
        "
      >
        <div style="display: flex; align-items: flex-start">
          <el-button
            type="primary"
            @click="selectFile()"
            :icon="FolderOpened"
            plain
          >
            Open File
          </el-button>
          <el-select v-model="data.sep" style="margin-left: 16px; width: 100px">
            <el-option label="," value="," />
            <el-option label="|" value="|" />
            <el-option label="\t" value="\t" />
            <el-option label=";" value=";" />
          </el-select>
          <el-button
            type="success"
            @click="accessData()"
            :loading="isLoading"
            :icon="Connection"
            plain
            style="margin-left: 16px"
          >
            Convert
          </el-button>
        </div>
        <el-text type="primary" size="large">
          <el-icon> <Connection /> </el-icon>
          Convert Access Database to CSV
        </el-text>
      </div>
    </el-form>
    <el-table
      ref="tableRef"
      :data="selectedFiles"
      :height="formHeight"
      style="width: 100%"
    >
      <el-table-column prop="filename" />
    </el-table>
  </el-form>
</template>

<style lang="scss">
.page-container {
  margin-bottom: 20px;
  padding: 20px;
  border-radius: 10px;
  background-color: #fff;
}
</style>
