<script setup lang="ts">
import { ref, reactive, computed, onMounted, onBeforeUnmount } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ElNotification } from "element-plus";
import {
  Cpu,
  FolderOpened,
  Loading,
  Select,
  CloseBold
} from "@element-plus/icons-vue";

const isLoading = ref(false);
const runtime = ref(0.0);
const progress = ref(0);
const tableRef = ref(null);
const selectedFiles = ref([]);
const data = reactive({
  filePath: "",
  fileFormats: ["csv", "txt", "tsv", "spext", "dat"],
  sep: ","
});
const windowHeight = ref(window.innerHeight);
const customColors = [
  { color: "#98FB98", percentage: 20 },
  { color: "#7CFC00", percentage: 40 },
  { color: "#7FFF00", percentage: 60 },
  { color: "#ADFF2F", percentage: 80 },
  { color: "#9ACD32", percentage: 100 }
];

const formHeight = computed(() => {
  const height = 225;
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

listen("start_convert", (event: any) => {
  const startConvert: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === startConvert) {
      file.status = "loading";
    }
  });
});
listen("runtime", (event: any) => {
  runtime.value = event.payload;
});
listen("drop_progress", (event: any) => {
  const pgs: any = event.payload;
  progress.value = pgs;
});
listen("drop_msg", (event: any) => {
  const dropMsg: any = event.payload;
  selectedFiles.value.forEach(file => {
    if (file.filename === dropMsg) {
      file.status = "completed";
    }
  });
});
listen("behead_err", (event: any) => {
  const fillErr = event.payload;
  ElNotification({
    title: "Behaed Error",
    message: fillErr,
    position: "bottom-right",
    type: "error",
    duration: 10000
  });
  isLoading.value = false;
});

// open file
async function selectFile() {
  selectedFiles.value = [];
  isLoading.value = false;
  progress.value = 0;

  const selected = await open({
    multiple: true,
    filters: [
      {
        name: "csv",
        extensions: data.fileFormats
      }
    ]
  });
  if (Array.isArray(selected)) {
    data.filePath = selected.join("|").toString();
    const nonEmptyRows = selected.filter((row: any) => row.trim() !== "");
    selectedFiles.value = nonEmptyRows.map((file: any) => {
      return { filename: file, status: "" };
    });
  } else if (selected === null) {
    return;
  } else {
    data.filePath = selected;
  }
}

// drop data
async function dropHeaders() {
  if (data.filePath === "") {
    ElNotification({
      title: "File not found",
      message: "未选择csv文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  if (data.filePath !== "") {
    isLoading.value = true;

    await invoke("behead", {
      filePath: data.filePath,
      sep: data.sep
    });

    ElNotification({
      message: "Drop done, elapsed time: " + runtime.value,
      position: "bottom-right",
      type: "success",
      duration: 10000
    });
    isLoading.value = false;
  }
}
</script>

<template>
  <el-form class="page-container" :style="formHeight">
    <div
      style="
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        position: sticky;
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
          @click="dropHeaders()"
          :loading="isLoading"
          :icon="Cpu"
          style="margin-left: 16px"
          plain
        >
          Drop
        </el-button>
      </div>
      <el-text type="primary" size="large">
        <el-icon> <Cpu /> </el-icon>
        <span>Drop headers from CSV</span>
      </el-text>
    </div>

    <el-table
      ref="tableRef"
      :data="selectedFiles"
      :height="formHeight"
      style="width: 100%"
    >
      <el-table-column prop="filename" label="file" style="width: 80%" />
      <el-table-column prop="status" label="status" width="100">
        <template #default="scope">
          <ElIcon v-if="scope.row.status === 'loading'" class="is-loading">
            <Loading />
          </ElIcon>
          <ElIcon v-else-if="scope.row.status === 'completed'" color="#00CD66">
            <Select />
          </ElIcon>
          <ElIcon v-else-if="scope.row.status === 'error'" color="#FF0000">
            <CloseBold />
          </ElIcon>
        </template>
      </el-table-column>
    </el-table>

    <el-progress
      v-if="isLoading"
      :percentage="progress"
      :color="customColors"
    />
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
