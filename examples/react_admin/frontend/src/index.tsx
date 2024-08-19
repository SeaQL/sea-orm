import ReactDOM from 'react-dom/client';
import { Admin, Resource, List, Datagrid, TextField, Show, SimpleShowLayout } from 'react-admin';
import { dataProvider } from './dataProvider';

const PostList = () => (
    <List>
        <Datagrid bulkActionButtons={false}>
            <TextField source="id" />
            <TextField source="title" />
            <TextField source="content" />
        </Datagrid>
    </List>
);

const PostShow = () => (
    <Show>
        <SimpleShowLayout>
            <TextField source="id" />
            <TextField source="title" />
            <TextField source="content" />
        </SimpleShowLayout>
    </Show>
);

ReactDOM.createRoot(document.getElementById('root')!).render(
    <Admin dataProvider={dataProvider}>
        <Resource name="posts" list={PostList} show={PostShow} />
    </Admin>
);
